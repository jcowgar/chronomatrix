//! Chronomatrix - A digital clock where each digit is composed of analog clocks.
//!
//! This is the main application entry point. It handles:
//! - GTK4 application initialization and window setup
//! - Configuration loading and hot-reload watching
//! - CSS styling with dynamic color injection
//! - Timer setup for clock updates every second
//!
//! The application creates a frameless window containing a `ClockDisplay` widget
//! that shows the current time as HH:MM:SS using 6 digits (each digit being
//! a 6x4 grid of 24 analog clocks).

mod analog_clock;
mod clock_display;
mod config;
mod digit_display;
mod digit_patterns;

use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, CssProvider, gdk, glib};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::cell::RefCell;
use std::collections::HashSet;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use clock_display::ClockDisplay;
use config::{Config, parse_hex_color};

const APP_ID: &str = "com.github.chronomatrix";

/// Debounce interval for config reload in milliseconds.
/// Prevents multiple reloads when a file is saved with multiple write events.
const CONFIG_RELOAD_DEBOUNCE_MS: u128 = 300;

/// Polling interval for checking the reload flag in milliseconds.
const CONFIG_POLL_INTERVAL_MS: u64 = 100;

/// Delay after detecting file change to ensure write is complete.
const FILE_WRITE_SETTLE_MS: u64 = 100;

/// State for managing config file reload with debouncing.
struct ReloadState {
    /// Whether a reload should be triggered
    should_reload: bool,
    /// Timestamp of the last reload to implement debouncing
    last_reload: std::time::Instant,
}

/// Holds the file watcher and the set of directories currently being watched.
/// Lives on the GTK thread so it can be mutated on reload.
struct WatcherState {
    watcher: RecommendedWatcher,
    watched_dirs: HashSet<PathBuf>,
}

/// Application entry point.
///
/// Creates a GTK4 application and runs it. The application is initialized with
/// the ID "com.github.chronomatrix" and connects the `build_ui` function to the
/// activate signal, which is triggered when the application starts.
fn main() -> glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(build_ui);

    app.run()
}

/// Builds and displays the main application UI.
///
/// This function:
/// - Loads the configuration from disk (or uses defaults)
/// - Creates a frameless window with the clock display
/// - Applies CSS styling with colors from config
/// - Sets up a 1-second timer for time updates
/// - Configures file watching for hot-reload of config changes
///
/// # Arguments
/// * `app` - The GTK application instance
fn build_ui(app: &Application) {
    // Load configuration (with include support)
    let load_result = Config::load_or_default();
    let source_files = load_result.source_files;
    let config = Rc::new(RefCell::new(load_result.config));

    // Create the main window
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Chronomatrix")
        .default_width(1200)
        .default_height(400)
        .decorated(false) // Remove title bar
        .build();

    // Load CSS for styling
    load_css(&config.borrow());

    // Create the clock display
    let clock_display = Rc::new(RefCell::new(ClockDisplay::new(&config.borrow())));

    // Set initial time
    clock_display.borrow().update_time();

    // Setup timer to update every second
    let clock_display_clone = clock_display.clone();
    glib::timeout_add_local(Duration::from_secs(1), move || {
        clock_display_clone.borrow().update_time();
        glib::ControlFlow::Continue
    });

    // Add the clock display to the window
    window.set_child(Some(clock_display.borrow().widget()));

    // Setup config file watcher (watches all source files including includes)
    setup_config_watcher(
        window.clone(),
        config.clone(),
        clock_display.clone(),
        source_files,
    );

    // Present the window
    window.present();
}

/// Resolves source file paths, canonicalizing where possible.
fn resolve_source_files(source_files: &[PathBuf]) -> HashSet<PathBuf> {
    source_files
        .iter()
        .map(|p| std::fs::canonicalize(p).unwrap_or_else(|_| p.clone()))
        .collect()
}

/// Computes the unique set of parent directories for a set of file paths.
fn parent_dirs(files: &HashSet<PathBuf>) -> HashSet<PathBuf> {
    files
        .iter()
        .filter_map(|p| p.parent().map(|d| d.to_path_buf()))
        .collect()
}

/// Sets up file system watching for configuration hot-reload.
///
/// Watches all source files (main config + includes). The watcher lives on the GTK
/// thread so it can be mutated on reload to watch/unwatch directories as includes change.
///
/// # Arguments
/// * `window` - The application window to update after reload
/// * `config` - Shared reference to the config that will be updated
/// * `clock_display` - Shared reference to the clock display to be recreated
/// * `source_files` - Initial set of config source files to watch
fn setup_config_watcher(
    window: ApplicationWindow,
    config: Rc<RefCell<Config>>,
    clock_display: Rc<RefCell<ClockDisplay>>,
    source_files: Vec<PathBuf>,
) {
    // Resolve all source files and compute directories to watch
    let resolved_files = resolve_source_files(&source_files);
    let dirs_to_watch = parent_dirs(&resolved_files);

    // Create a channel for file system events
    let (tx, rx) = channel();

    // Create watcher on GTK thread
    let mut watcher = match notify::recommended_watcher(tx) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("Failed to create file watcher: {}", e);
            return;
        }
    };

    // Watch all directories containing source files
    let mut watched_dirs = HashSet::new();
    for dir in &dirs_to_watch {
        if let Err(e) = watcher.watch(dir, RecursiveMode::NonRecursive) {
            eprintln!("Failed to watch directory {:?}: {}", dir, e);
        } else {
            watched_dirs.insert(dir.clone());
        }
    }

    let watcher_state = Rc::new(RefCell::new(WatcherState {
        watcher,
        watched_dirs,
    }));

    // Shared set of watched file paths (accessed by background thread for filtering)
    let watched_files: Arc<Mutex<HashSet<PathBuf>>> = Arc::new(Mutex::new(resolved_files));
    let watched_files_for_thread = watched_files.clone();

    // Create a state struct to signal config reload with timestamp for debouncing
    let reload_state = Arc::new(Mutex::new(ReloadState {
        should_reload: false,
        last_reload: std::time::Instant::now(),
    }));
    let reload_state_clone = reload_state.clone();

    // Spawn background thread to receive events and set the reload flag
    thread::spawn(move || {
        loop {
            match rx.recv() {
                Ok(Ok(event)) => {
                    if is_watched_file_event(&event, &watched_files_for_thread) {
                        // Small delay to ensure file is written completely
                        thread::sleep(Duration::from_millis(FILE_WRITE_SETTLE_MS));

                        if let Ok(mut state) = reload_state_clone.lock() {
                            let now = std::time::Instant::now();
                            if now.duration_since(state.last_reload).as_millis()
                                > CONFIG_RELOAD_DEBOUNCE_MS
                            {
                                state.should_reload = true;
                                state.last_reload = now;
                            }
                        }
                    }
                }
                Ok(Err(e)) => eprintln!("Watch error: {:?}", e),
                Err(e) => {
                    eprintln!("Channel error: {:?}", e);
                    break;
                }
            }
        }
    });

    // Poll the reload flag in the GTK main loop
    glib::timeout_add_local(Duration::from_millis(CONFIG_POLL_INTERVAL_MS), move || {
        if let Ok(mut state) = reload_state.lock()
            && state.should_reload
        {
            state.should_reload = false;
            reload_config(
                &window,
                &config,
                &clock_display,
                &watcher_state,
                &watched_files,
            );
        }
        glib::ControlFlow::Continue
    });
}

/// Checks if a file system event involves any of the watched config files.
///
/// Tries to match by canonicalized path first, falls back to matching by filename.
fn is_watched_file_event(
    event: &notify::Event,
    watched_files: &Arc<Mutex<HashSet<PathBuf>>>,
) -> bool {
    let files = match watched_files.lock() {
        Ok(f) => f,
        Err(_) => return false,
    };

    event.paths.iter().any(|event_path| {
        // Try exact match via canonicalization
        if let Ok(canonical) = std::fs::canonicalize(event_path) {
            if files.contains(&canonical) {
                return true;
            }
        }
        // Fallback: match by filename against any watched file
        if let Some(event_name) = event_path.file_name() {
            return files.iter().any(|f| f.file_name() == Some(event_name));
        }
        false
    })
}

/// Reloads the configuration, recreates the clock display, and updates watched files/dirs.
fn reload_config(
    window: &ApplicationWindow,
    config: &Rc<RefCell<Config>>,
    clock_display: &Rc<RefCell<ClockDisplay>>,
    watcher_state: &Rc<RefCell<WatcherState>>,
    watched_files: &Arc<Mutex<HashSet<PathBuf>>>,
) {
    // Load new config (with includes)
    let load_result = Config::load_or_default();
    let new_config = load_result.config;

    // Reload CSS
    load_css(&new_config);

    // Store the new config
    *config.borrow_mut() = new_config.clone();

    // Recreate the clock display with new config
    let new_clock_display = ClockDisplay::new(&new_config);
    new_clock_display.update_time_immediate();
    window.set_child(Some(new_clock_display.widget()));
    *clock_display.borrow_mut() = new_clock_display;

    // Update watched files and directories
    let new_resolved = resolve_source_files(&load_result.source_files);
    let new_dirs = parent_dirs(&new_resolved);

    let mut ws = watcher_state.borrow_mut();
    let old_dirs = ws.watched_dirs.clone();

    // Unwatch directories that are no longer needed
    for dir in old_dirs.difference(&new_dirs) {
        if let Err(e) = ws.watcher.unwatch(dir) {
            eprintln!("Failed to unwatch {:?}: {}", dir, e);
        }
    }

    // Watch new directories
    for dir in new_dirs.difference(&old_dirs) {
        if let Err(e) = ws.watcher.watch(dir, RecursiveMode::NonRecursive) {
            eprintln!("Failed to watch {:?}: {}", dir, e);
        }
    }

    ws.watched_dirs = new_dirs;

    // Update shared file set
    if let Ok(mut files) = watched_files.lock() {
        *files = new_resolved;
    }
}

/// Loads and applies CSS styling based on configuration.
///
/// Generates CSS rules dynamically from config colors and applies them to
/// the GTK display. This handles:
/// - Window background color with opacity
/// - Display container styling (background, border, padding, border-radius)
///
/// The window opacity setting from config is applied to the background color's
/// alpha channel, allowing transparent backgrounds while keeping UI elements opaque.
///
/// # Arguments
/// * `config` - The configuration containing color and opacity settings
fn load_css(config: &Config) {
    let provider = CssProvider::new();

    // Parse background color and apply window opacity to it
    let bg_color = &config.colors.window_background;
    let (r, g, b, a) = parse_hex_color(bg_color);
    // Multiply alpha by the opacity setting to control background transparency
    let final_alpha = a * config.window.opacity;
    let bg_rgba = format!(
        "rgba({}, {}, {}, {})",
        (r * 255.0) as u8,
        (g * 255.0) as u8,
        (b * 255.0) as u8,
        final_alpha
    );

    // Parse display colors
    let display_bg = &config.colors.display_bg;
    let (dr, dg, db, da) = parse_hex_color(display_bg);
    let display_rgba = format!(
        "rgba({}, {}, {}, {})",
        (dr * 255.0) as u8,
        (dg * 255.0) as u8,
        (db * 255.0) as u8,
        da
    );

    let display_border = &config.colors.display_border;
    let (border_r, border_g, border_b, border_a) = parse_hex_color(display_border);
    let border_rgba = format!(
        "rgba({}, {}, {}, {})",
        (border_r * 255.0) as u8,
        (border_g * 255.0) as u8,
        (border_b * 255.0) as u8,
        border_a
    );

    let css = format!(
        r#"
        window {{
            background-color: {bg_rgba};
            background: {bg_rgba};
        }}

        .clock-display {{
            padding: 40px;
            background: {display_rgba};
            border-radius: 20px;
            border: 1px solid {border_rgba};
        }}
        "#
    );

    provider.load_from_string(&css);

    gtk4::style_context_add_provider_for_display(
        &gdk::Display::default().expect("Could not connect to a display."),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
