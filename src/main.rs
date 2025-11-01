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
use notify::{Event, RecursiveMode, Watcher};
use std::cell::RefCell;
use std::path::Path;
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
    // Load configuration
    let config = Rc::new(RefCell::new(Config::load_or_default()));

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

    // Setup config file watcher
    setup_config_watcher(window.clone(), config.clone(), clock_display.clone());

    // Present the window
    window.present();
}

/// Sets up file system watching for configuration hot-reload.
///
/// Creates a background thread that monitors the config directory for file changes.
/// When the config file is modified, it triggers a reload in the GTK main loop.
/// Uses debouncing (300ms) to avoid multiple reloads from rapid file changes.
///
/// # How it works
/// 1. Spawns a background thread with a file system watcher
/// 2. Watches the config directory for any modifications
/// 3. Filters events to only config file changes
/// 4. Sets a reload flag (with timestamp for debouncing)
/// 5. GTK main loop polls the flag every 100ms and triggers reload
///
/// # Arguments
/// * `window` - The application window to update after reload
/// * `config` - Shared reference to the config that will be updated
/// * `clock_display` - Shared reference to the clock display to be recreated
fn setup_config_watcher(
    window: ApplicationWindow,
    config: Rc<RefCell<Config>>,
    clock_display: Rc<RefCell<ClockDisplay>>,
) {
    let config_path = Config::default_path();

    // Get the directory to watch
    let watch_dir = config_path.parent().map(|p| p.to_path_buf());

    if watch_dir.is_none() {
        eprintln!("Could not determine config directory to watch");
        return;
    }

    let watch_dir = watch_dir.unwrap();

    // Create a channel for file system events
    let (tx, rx) = channel();

    // Create a state struct to signal config reload with timestamp for debouncing
    let reload_state = Arc::new(Mutex::new(ReloadState {
        should_reload: false,
        last_reload: std::time::Instant::now(),
    }));
    let reload_state_clone = reload_state.clone();

    // Spawn a thread to watch the config file
    thread::spawn(move || {
        let mut watcher = match notify::recommended_watcher(tx) {
            Ok(w) => w,
            Err(e) => {
                eprintln!("Failed to create file watcher: {}", e);
                return;
            }
        };

        // Watch the config directory
        if let Err(e) = watcher.watch(&watch_dir, RecursiveMode::NonRecursive) {
            eprintln!("Failed to watch config directory: {}", e);
            return;
        }

        // Keep the watcher alive and forward events
        loop {
            match rx.recv() {
                Ok(Ok(event)) => {
                    // Check if the config file was modified
                    if is_config_file_event(&event, &config_path) {
                        // Small delay to ensure file is written completely
                        thread::sleep(Duration::from_millis(FILE_WRITE_SETTLE_MS));

                        // Set the reload flag with debouncing
                        if let Ok(mut state) = reload_state_clone.lock() {
                            let now = std::time::Instant::now();
                            // Only set flag if enough time has passed since last reload (debouncing)
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
            reload_config(&window, &config, &clock_display);
        }
        glib::ControlFlow::Continue
    });
}

/// Checks if a file system event is related to the config file.
///
/// Compares the file name from the event against the config file name.
///
/// # Arguments
/// * `event` - The file system event to check
/// * `config_path` - Path to the config file
///
/// # Returns
/// `true` if the event involves the config file, `false` otherwise
fn is_config_file_event(event: &Event, config_path: &Path) -> bool {
    event
        .paths
        .iter()
        .any(|path| path.file_name() == config_path.file_name())
}

/// Reloads the configuration and recreates the clock display.
///
/// This function:
/// 1. Loads the new configuration from disk
/// 2. Reapplies CSS with updated colors
/// 3. Creates a new clock display with the new settings
/// 4. Replaces the window content with the new display
/// 5. Sets the time immediately (without animation) for accurate display
///
/// # Arguments
/// * `window` - The application window to update
/// * `config` - Shared reference to store the new config
/// * `clock_display` - Shared reference to store the new clock display
fn reload_config(
    window: &ApplicationWindow,
    config: &Rc<RefCell<Config>>,
    clock_display: &Rc<RefCell<ClockDisplay>>,
) {
    // Load new config
    let new_config = Config::load_or_default();

    // Reload CSS (this handles background opacity via CSS colors)
    load_css(&new_config);

    // Store the new config
    *config.borrow_mut() = new_config.clone();

    // Recreate the clock display with new config
    let new_clock_display = ClockDisplay::new(&new_config);
    // Use immediate update to set correct angles without animation
    new_clock_display.update_time_immediate();

    // Replace the old clock display widget
    window.set_child(Some(new_clock_display.widget()));

    // Update the Rc to point to new display
    *clock_display.borrow_mut() = new_clock_display;
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
