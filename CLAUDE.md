# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Chronomatrix - A creative digital clock where each digit (0-9) is composed of a 6x4 grid of 24 individual analog clocks. The clock hands on each mini analog clock are positioned to form the shape of the digit. Built with Rust, GTK4, libadwaita, and Cairo for rendering.

## Development Environment

### With Nix (Recommended)
```bash
nix develop              # Enter development environment
cargo build --release    # Build the project
cargo run --release      # Run the application
```

### Without Nix
Requires: Rust 1.82+ (edition 2024), GTK4, libadwaita, Cairo dev libraries, pkg-config

```bash
cargo build --release
cargo run --release
```

## Architecture

### Module Structure

The codebase follows a clear separation of concerns across 5 modules:

1. **`main.rs`** - Application bootstrap and GTK window setup
   - Initializes GTK4 application with ID `com.github.chronomatrix`
   - Loads config and applies CSS styling dynamically based on config colors
   - Sets up 1-second timer for clock updates

2. **`config.rs`** - Configuration management
   - TOML-based config loaded from platform-specific paths:
     - Linux: `~/.config/chronomatrix/config.toml`
     - macOS: `~/Library/Application Support/chronomatrix/config.toml`
     - Windows: `%APPDATA%\chronomatrix\config.toml`
   - `parse_hex_color()` converts hex colors (#RRGGBB or #RRGGBBAA) to Cairo RGBA values (0.0-1.0)
   - Provides sensible defaults if config file is missing

3. **`digit_patterns.rs`** - Digit shape definitions
   - Each digit (0-9) defined as a `DigitPattern` (6x4 array of `ClockPosition` structs)
   - `ClockPosition` stores hour/minute hand angles in degrees (0-359)
   - Inactive clocks use `INACTIVE` constant (hour=135°, minute=315°) for diagonal appearance
   - All 10 digit patterns are pre-defined as compile-time constants

4. **`digit_display.rs`** - 6x4 grid of analog clocks for a single digit
   - Creates a GTK Grid containing 24 `AnalogClock` instances
   - `set_digit(u8)` looks up the pattern and applies angles to all 24 clocks

5. **`clock_display.rs`** - Overall HH:MM:SS display
   - Creates 6 `DigitDisplay` instances (for hours, minutes, seconds)
   - Adds separator dots between digit pairs (`:`)
   - `update_time()` uses chrono to get current time and updates all digits

6. **`analog_clock.rs`** - Individual analog clock widget (GTK DrawingArea subclass)
   - Custom GTK widget using GObject subclassing pattern
   - Cairo-based rendering of clock face, border, and two hands
   - Smooth animations at 60 FPS with ease-in-out easing
   - Cumulative angle tracking ensures hands always rotate clockwise
   - Active/inactive state determines hand color (active vs. inactive with transparency)

### Key Technical Patterns

**Animation System:**
- `set_angles()` triggers animation by setting target cumulative angles
- 60 FPS timer (16ms interval) in `start_animation_loop()`
- `update_animation()` interpolates between start and target using ease-in-out curve
- Cumulative angle tracking prevents backwards rotation (e.g., 350° → 10° rotates +20° not -340°)

**Color System:**
- All colors stored as hex strings in TOML config
- `parse_hex_color()` converts to Cairo's 0.0-1.0 RGBA format
- Supports alpha channel for transparency effects
- Colors injected into GTK CSS and passed to Cairo rendering

**GTK4 Custom Widget Pattern:**
- `AnalogClock` uses GObject subclassing (`glib::wrapper!`, `ObjectSubclass`)
- Internal state in `imp::AnalogClock` accessed via `RefCell` for interior mutability
- `set_draw_func()` registers Cairo drawing callback

## Configuration

The application is highly configurable via `config.toml`:
- All visual aspects (colors, sizes, gaps, animation speed) can be customized
- See `config.toml` in project root for all available options and examples
- Config changes require application restart

## Code Style Notes

- Uses Rust edition 2024
- GTK widgets created using builder pattern or `::new()` constructors
- Clone semantics: `ClockColors` is `Clone` for sharing across widgets
- Rc<RefCell<>> pattern used for shared mutable state (separator colors, clock display)
- Type safety: Angles stored as `i32` degrees, converted to `f64` radians for Cairo
