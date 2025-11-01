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
use std::path::PathBuf;

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
    /// Loads configuration from a TOML file.
    ///
    /// Reads and parses a TOML configuration file into a `Config` struct.
    /// All fields use `#[serde(default)]`, so partial configs are supported.
    ///
    /// # Arguments
    /// * `path` - Path to the TOML configuration file
    ///
    /// # Returns
    /// * `Ok(Config)` - Successfully loaded configuration
    /// * `Err` - File not found, permission denied, or invalid TOML syntax
    ///
    /// # Example
    /// ```no_run
    /// use std::path::PathBuf;
    /// # use chronomatrix::config::Config;
    /// let path = PathBuf::from("~/.config/chronomatrix/config.toml");
    /// let config = Config::load(&path)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn load(path: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
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
    ///
    /// # Returns
    /// PathBuf pointing to the default config location
    pub fn default_path() -> PathBuf {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("chronomatrix");
        path.push("config.toml");
        path
    }

    /// Loads config from the default path, falling back to defaults if unavailable.
    ///
    /// Attempts to load the config file from `default_path()`. If the file doesn't
    /// exist, is unreadable, or contains invalid TOML, prints a warning to stderr
    /// and returns `Config::default()` instead.
    ///
    /// This is the recommended way to load configuration in the application, as it
    /// gracefully handles missing or invalid config files.
    ///
    /// # Returns
    /// The loaded configuration, or default configuration if loading fails
    pub fn load_or_default() -> Self {
        let path = Self::default_path();
        Self::load(&path).unwrap_or_else(|_| {
            eprintln!("Could not load config from {:?}, using defaults", path);
            Self::default()
        })
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
}
