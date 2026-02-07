//! Configuration management for Chronomatrix.
//!
//! This module handles loading and parsing TOML configuration files from
//! platform-specific directories. It provides:
//! - Structured configuration with sensible defaults
//! - TOML deserialization with `#[serde(default)]` for graceful partial configs
//! - Hex color parsing (#RRGGBB or #RRGGBBAA) to Cairo RGBA values
//!
//! # Configuration Location
//! - Linux: `~/.config/chronomatrix/config.toml`
//! - macOS: `~/Library/Application Support/chronomatrix/config.toml`
//! - Windows: `%APPDATA%\chronomatrix\config.toml`
//!
//! # Example
//! ```toml
//! [colors]
//! clock_hand_color = "#ff6b6b"
//! clock_hand_inactive = "#ff6b6b26"
//!
//! [window]
//! opacity = 0.95
//!
//! [clock]
//! size = 45
//! animation_duration_ms = 400
//! ```

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use toml::value::Table;

/// Result of loading configuration, including all source file paths for hot-reload watching.
pub struct ConfigLoadResult {
    pub config: Config,
    /// All resolved file paths that contributed to this config (main + includes).
    pub source_files: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    #[serde(default)]
    pub colors: ColorConfig,
    #[serde(default)]
    pub window: WindowConfig,
    #[serde(default)]
    pub clock: ClockConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ColorConfig {
    /// Window background color (hex format: #RRGGBB or #RRGGBBAA)
    pub window_background: String,
    /// Active clock hand color (hex format: #RRGGBB or #RRGGBBAA)
    pub clock_hand_color: String,
    /// Inactive clock hand color (hex format: #RRGGBB or #RRGGBBAA)
    pub clock_hand_inactive: String,
    /// Clock background color (hex format: #RRGGBB or #RRGGBBAA)
    pub clock_bg: String,
    /// Clock border color (hex format: #RRGGBB or #RRGGBBAA)
    pub clock_border: String,
    /// Display container background color (hex format: #RRGGBB or #RRGGBBAA)
    pub display_bg: String,
    /// Display container border color (hex format: #RRGGBB or #RRGGBBAA)
    pub display_border: String,
    /// Separator dot color (hex format: #RRGGBB or #RRGGBBAA)
    pub separator_color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WindowConfig {
    /// Enable window transparency (deprecated, kept for compatibility)
    #[serde(default)]
    pub transparent: bool,
    /// Window opacity (0.0 - 1.0)
    pub opacity: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ClockConfig {
    /// Size of each individual analog clock in pixels
    pub size: i32,
    /// Stroke width for clock hands
    pub stroke_width: f64,
    /// Gap between clocks in pixels
    pub clock_gap: i32,
    /// Gap between digit sections
    pub digit_gap: i32,
    /// Animation duration in milliseconds for hand rotation
    pub animation_duration_ms: u64,
}

impl Default for ColorConfig {
    fn default() -> Self {
        ColorConfig {
            window_background: "#0f0c29".to_string(),
            clock_hand_color: "#ff6b6b".to_string(),
            clock_hand_inactive: "#ff6b6b26".to_string(), // 15% opacity
            clock_bg: "#ffffff08".to_string(),            // 3% opacity
            clock_border: "#ffffff1a".to_string(),        // 10% opacity
            display_bg: "#ffffff0d".to_string(),          // 5% opacity
            display_border: "#ffffff1a".to_string(),      // 10% opacity
            separator_color: "#ff6b6b".to_string(),
        }
    }
}

impl Default for WindowConfig {
    fn default() -> Self {
        WindowConfig {
            transparent: false,
            opacity: 1.0,
        }
    }
}

impl Default for ClockConfig {
    fn default() -> Self {
        ClockConfig {
            size: 40,
            stroke_width: 2.0,
            clock_gap: 1,
            digit_gap: 8,
            animation_duration_ms: 300,
        }
    }
}

impl Config {
    /// Loads configuration from a TOML file, processing any `include` directives.
    ///
    /// The `include` key is an array of file paths (relative to the main config's
    /// directory or absolute) that are loaded and deep-merged on top of the main
    /// config. Later includes override earlier ones. Included files' own `include`
    /// keys are stripped (no recursive includes).
    ///
    /// # Arguments
    /// * `path` - Path to the TOML configuration file
    ///
    /// # Returns
    /// * `Ok(ConfigLoadResult)` - Config and all source file paths
    /// * `Err` - File not found, permission denied, or invalid TOML syntax
    pub fn load(path: &PathBuf) -> Result<ConfigLoadResult, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let mut table: Table = toml::from_str(&contents)?;

        let base_dir = path.parent().unwrap_or(Path::new("."));
        let mut source_files = vec![path.clone()];

        // Extract and remove the `include` array before deserialization
        if let Some(include_val) = table.remove("include") {
            if let Some(includes) = include_val.as_array() {
                for item in includes {
                    if let Some(include_str) = item.as_str() {
                        let include_path = resolve_include_path(base_dir, include_str);
                        let canonical = match fs::canonicalize(&include_path) {
                            Ok(p) => p,
                            Err(e) => {
                                eprintln!(
                                    "Warning: Could not resolve include path {:?}: {}",
                                    include_path, e
                                );
                                continue;
                            }
                        };

                        match fs::read_to_string(&canonical) {
                            Ok(inc_contents) => match toml::from_str::<Table>(&inc_contents) {
                                Ok(mut inc_table) => {
                                    // Strip any nested include keys (no recursive includes)
                                    inc_table.remove("include");
                                    deep_merge_toml(&mut table, inc_table);
                                    source_files.push(canonical);
                                }
                                Err(e) => {
                                    eprintln!(
                                        "Warning: Failed to parse include file {:?}: {}",
                                        canonical, e
                                    );
                                }
                            },
                            Err(e) => {
                                eprintln!(
                                    "Warning: Could not read include file {:?}: {}",
                                    canonical, e
                                );
                            }
                        }
                    }
                }
            }
        }

        let config: Config = toml::Value::Table(table).try_into()?;
        Ok(ConfigLoadResult {
            config,
            source_files,
        })
    }

    /// Returns the platform-specific default path for the config file.
    ///
    /// # Platform Paths
    /// - **Linux**: `~/.config/chronomatrix/config.toml`
    /// - **macOS**: `~/Library/Application Support/chronomatrix/config.toml`
    /// - **Windows**: `%APPDATA%\chronomatrix\config.toml`
    ///
    /// Falls back to `./chronomatrix/config.toml` if the system config directory
    /// cannot be determined.
    pub fn default_path() -> PathBuf {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("chronomatrix");
        path.push("config.toml");
        path
    }

    /// Loads config from the default path, falling back to defaults if unavailable.
    ///
    /// Returns a `ConfigLoadResult` containing the config and all source file paths.
    /// If loading fails, returns default config with just the default path as source.
    pub fn load_or_default() -> ConfigLoadResult {
        let path = Self::default_path();
        Self::load(&path).unwrap_or_else(|_| {
            eprintln!("Could not load config from {:?}, using defaults", path);
            ConfigLoadResult {
                config: Self::default(),
                source_files: vec![path],
            }
        })
    }
}

/// Resolves an include path relative to a base directory.
///
/// If the include path is absolute, it is returned as-is.
/// If relative, it is joined to the base directory.
pub fn resolve_include_path(base_dir: &Path, include: &str) -> PathBuf {
    let path = Path::new(include);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        base_dir.join(path)
    }
}

/// Deep-merges an overlay TOML table into a base table.
///
/// For nested tables, merges recursively. For all other value types,
/// the overlay value replaces the base value.
pub fn deep_merge_toml(base: &mut Table, overlay: Table) {
    for (key, overlay_val) in overlay {
        match (base.get_mut(&key), overlay_val.clone()) {
            (Some(toml::Value::Table(base_table)), toml::Value::Table(overlay_table)) => {
                deep_merge_toml(base_table, overlay_table);
            }
            _ => {
                base.insert(key, overlay_val);
            }
        }
    }
}

/// Parses a hex color string into Cairo-compatible RGBA values.
///
/// Converts hex color strings (with or without alpha) into normalized RGBA
/// values in the range 0.0-1.0, suitable for use with Cairo rendering.
///
/// # Supported Formats
/// - `#RRGGBB` - 6 hex digits (RGB), alpha defaults to 1.0 (fully opaque)
/// - `#RRGGBBAA` - 8 hex digits (RGBA), alpha specified
///
/// Leading `#` is optional. Invalid hex digits default to 0.
/// Unsupported lengths default to black: `(0.0, 0.0, 0.0, 1.0)`.
///
/// # Arguments
/// * `hex` - Hex color string (e.g., "#ff6b6b" or "#ff6b6b26")
///
/// # Returns
/// Tuple of `(red, green, blue, alpha)` where each value is 0.0-1.0
///
/// # Examples
/// ```
/// # use chronomatrix::config::parse_hex_color;
/// // RGB color
/// let red = parse_hex_color("#ff0000");
/// assert_eq!(red, (1.0, 0.0, 0.0, 1.0));
///
/// // RGBA with 15% opacity (0x26 = 38 / 255 ≈ 0.149)
/// let semi_transparent = parse_hex_color("#ff6b6b26");
/// assert_eq!(semi_transparent.3, 38.0 / 255.0);
/// ```
pub fn parse_hex_color(hex: &str) -> (f64, f64, f64, f64) {
    let hex = hex.trim_start_matches('#');

    let (r, g, b, a) = match hex.len() {
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or_else(|_| {
                eprintln!(
                    "Warning: Invalid red component in hex color '{}', using 0",
                    hex
                );
                0
            });
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or_else(|_| {
                eprintln!(
                    "Warning: Invalid green component in hex color '{}', using 0",
                    hex
                );
                0
            });
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or_else(|_| {
                eprintln!(
                    "Warning: Invalid blue component in hex color '{}', using 0",
                    hex
                );
                0
            });
            (r, g, b, 255)
        }
        8 => {
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or_else(|_| {
                eprintln!(
                    "Warning: Invalid red component in hex color '{}', using 0",
                    hex
                );
                0
            });
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or_else(|_| {
                eprintln!(
                    "Warning: Invalid green component in hex color '{}', using 0",
                    hex
                );
                0
            });
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or_else(|_| {
                eprintln!(
                    "Warning: Invalid blue component in hex color '{}', using 0",
                    hex
                );
                0
            });
            let a = u8::from_str_radix(&hex[6..8], 16).unwrap_or_else(|_| {
                eprintln!(
                    "Warning: Invalid alpha component in hex color '{}', using 255",
                    hex
                );
                255
            });
            (r, g, b, a)
        }
        _ => {
            eprintln!(
                "Warning: Invalid hex color format '{}' (expected 6 or 8 characters), using black",
                hex
            );
            (0, 0, 0, 255)
        }
    };

    (
        r as f64 / 255.0,
        g as f64 / 255.0,
        b as f64 / 255.0,
        a as f64 / 255.0,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_color_rgb() {
        // Test standard RGB format
        let (r, g, b, a) = parse_hex_color("#ff0000");
        assert_eq!(r, 1.0);
        assert_eq!(g, 0.0);
        assert_eq!(b, 0.0);
        assert_eq!(a, 1.0);
    }

    #[test]
    fn test_parse_hex_color_rgba() {
        // Test RGBA format with transparency
        let (r, g, b, a) = parse_hex_color("#ff6b6b26");
        assert_eq!(r, 1.0);
        assert!((g - 0.4196).abs() < 0.01); // 0x6b = 107 / 255 ≈ 0.4196
        assert!((b - 0.4196).abs() < 0.01);
        assert!((a - 0.1490).abs() < 0.01); // 0x26 = 38 / 255 ≈ 0.149
    }

    #[test]
    fn test_parse_hex_color_no_hash() {
        // Test without leading #
        let (r, g, b, a) = parse_hex_color("00ff00");
        assert_eq!(r, 0.0);
        assert_eq!(g, 1.0);
        assert_eq!(b, 0.0);
        assert_eq!(a, 1.0);
    }

    #[test]
    fn test_parse_hex_color_black() {
        let (r, g, b, a) = parse_hex_color("#000000");
        assert_eq!(r, 0.0);
        assert_eq!(g, 0.0);
        assert_eq!(b, 0.0);
        assert_eq!(a, 1.0);
    }

    #[test]
    fn test_parse_hex_color_white() {
        let (r, g, b, a) = parse_hex_color("#ffffff");
        assert_eq!(r, 1.0);
        assert_eq!(g, 1.0);
        assert_eq!(b, 1.0);
        assert_eq!(a, 1.0);
    }

    #[test]
    fn test_parse_hex_color_fully_transparent() {
        let (r, g, b, a) = parse_hex_color("#ff000000");
        assert_eq!(r, 1.0);
        assert_eq!(g, 0.0);
        assert_eq!(b, 0.0);
        assert_eq!(a, 0.0);
    }

    #[test]
    fn test_parse_hex_color_invalid_length() {
        // Invalid length should default to black
        let (r, g, b, a) = parse_hex_color("#fff");
        assert_eq!(r, 0.0);
        assert_eq!(g, 0.0);
        assert_eq!(b, 0.0);
        assert_eq!(a, 1.0);
    }

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.colors.window_background, "#0f0c29");
        assert_eq!(config.window.opacity, 1.0);
        assert_eq!(config.clock.size, 40);
        assert_eq!(config.clock.animation_duration_ms, 300);
    }

    #[test]
    fn test_config_default_path() {
        let path = Config::default_path();
        let path_str = path.to_string_lossy();
        assert!(path_str.contains("chronomatrix"));
        assert!(path_str.ends_with("config.toml"));
    }

    #[test]
    fn test_deep_merge_toml_simple_override() {
        let mut base: Table = toml::from_str(r#"key = "base""#).unwrap();
        let overlay: Table = toml::from_str(r#"key = "overlay""#).unwrap();
        deep_merge_toml(&mut base, overlay);
        assert_eq!(base["key"].as_str().unwrap(), "overlay");
    }

    #[test]
    fn test_deep_merge_toml_nested_tables() {
        let mut base: Table = toml::from_str(
            r##"
            [colors]
            window_background = "#000000"
            clock_hand_color = "#ff0000"
            "##,
        )
        .unwrap();
        let overlay: Table = toml::from_str(
            r##"
            [colors]
            window_background = "#ffffff"
            "##,
        )
        .unwrap();
        deep_merge_toml(&mut base, overlay);

        let colors = base["colors"].as_table().unwrap();
        assert_eq!(colors["window_background"].as_str().unwrap(), "#ffffff");
        // Original key should be preserved
        assert_eq!(colors["clock_hand_color"].as_str().unwrap(), "#ff0000");
    }

    #[test]
    fn test_deep_merge_toml_adds_new_keys() {
        let mut base: Table = toml::from_str(r#"a = 1"#).unwrap();
        let overlay: Table = toml::from_str(r#"b = 2"#).unwrap();
        deep_merge_toml(&mut base, overlay);
        assert_eq!(base["a"].as_integer().unwrap(), 1);
        assert_eq!(base["b"].as_integer().unwrap(), 2);
    }

    #[test]
    fn test_deep_merge_toml_overlay_table_replaces_scalar() {
        let mut base: Table = toml::from_str(r#"key = "scalar""#).unwrap();
        let overlay: Table = toml::from_str(
            r#"
            [key]
            nested = "value"
            "#,
        )
        .unwrap();
        deep_merge_toml(&mut base, overlay);
        assert!(base["key"].is_table());
        assert_eq!(
            base["key"].as_table().unwrap()["nested"].as_str().unwrap(),
            "value"
        );
    }

    #[test]
    fn test_resolve_include_path_relative() {
        let base = Path::new("/home/user/.config/chronomatrix");
        let result = resolve_include_path(base, "theme.toml");
        assert_eq!(
            result,
            PathBuf::from("/home/user/.config/chronomatrix/theme.toml")
        );
    }

    #[test]
    fn test_resolve_include_path_absolute() {
        let base = Path::new("/home/user/.config/chronomatrix");
        let result = resolve_include_path(base, "/etc/chronomatrix/theme.toml");
        assert_eq!(result, PathBuf::from("/etc/chronomatrix/theme.toml"));
    }

    #[test]
    fn test_load_with_includes() {
        use std::io::Write;
        let dir = std::env::temp_dir().join("chronomatrix_test_includes");
        let _ = fs::create_dir_all(&dir);

        let main_config = dir.join("config.toml");
        let theme_file = dir.join("theme.toml");

        let mut f = fs::File::create(&theme_file).unwrap();
        writeln!(
            f,
            r##"
[colors]
window_background = "#abcdef"
"##
        )
        .unwrap();

        let mut f = fs::File::create(&main_config).unwrap();
        writeln!(
            f,
            r##"
include = ["theme.toml"]

[colors]
clock_hand_color = "#112233"
"##
        )
        .unwrap();

        let result = Config::load(&main_config).unwrap();
        assert_eq!(result.config.colors.window_background, "#abcdef");
        assert_eq!(result.config.colors.clock_hand_color, "#112233");
        assert_eq!(result.source_files.len(), 2);

        let _ = fs::remove_dir_all(&dir);
    }
}
