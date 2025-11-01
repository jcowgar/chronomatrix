//! Overall HH:MM:SS clock display.
//!
//! This module creates the complete clock display showing the current time
//! in HH:MM:SS format. It manages:
//! - 6 `DigitDisplay` widgets (2 for hours, 2 for minutes, 2 for seconds)
//! - Separator dots (`:`) between digit pairs
//! - Time updates using the system clock via chrono
//!
//! The layout is: `[HH] : [MM] : [SS]` where each digit is a 6x4 grid
//! of 24 analog clocks.

use chrono::Local;
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, DrawingArea, Orientation, Widget, glib};
use std::cell::RefCell;
use std::f64::consts::PI;
use std::rc::Rc;

use crate::analog_clock::ClockColors;
use crate::config::{Config, parse_hex_color};
use crate::digit_display::DigitDisplay;

/// Width of the separator area containing the colon dots
const SEPARATOR_WIDTH: i32 = 20;

/// Height of the separator area
const SEPARATOR_HEIGHT: i32 = 200;

/// Y position of the top dot in the separator
const SEPARATOR_TOP_DOT_Y: f64 = 60.0;

/// Y position of the bottom dot in the separator
const SEPARATOR_BOTTOM_DOT_Y: f64 = 140.0;

/// Radius of the separator dots
const SEPARATOR_DOT_RADIUS: f64 = 4.0;

/// Gap between digit groups (hours, minutes, seconds)
const DIGIT_GROUP_GAP: i32 = 20;

pub struct ClockDisplay {
    container: GtkBox,
    digits: Vec<DigitDisplay>,
}

impl ClockDisplay {
    /// Creates a new clock display showing HH:MM:SS.
    ///
    /// Constructs the complete time display with 6 digits (2 each for hours,
    /// minutes, and seconds) arranged horizontally with separator dots between
    /// each pair.
    ///
    /// # Arguments
    /// * `config` - Configuration containing colors, sizes, and animation settings
    ///
    /// # Returns
    /// A new `ClockDisplay` ready to be added to a GTK container
    pub fn new(config: &Config) -> Self {
        let container = GtkBox::new(Orientation::Horizontal, DIGIT_GROUP_GAP);
        container.set_halign(gtk4::Align::Center);
        container.set_valign(gtk4::Align::Center);

        // Apply display styling
        container.add_css_class("clock-display");

        let clock_colors = ClockColors {
            active_color: parse_hex_color(&config.colors.clock_hand_color),
            inactive_color: parse_hex_color(&config.colors.clock_hand_inactive),
            bg_color: parse_hex_color(&config.colors.clock_bg),
            border_color: parse_hex_color(&config.colors.clock_border),
        };

        let separator_color = Rc::new(RefCell::new(parse_hex_color(
            &config.colors.separator_color,
        )));

        let mut digits = Vec::new();

        // Create 6 digits (HH:MM:SS)
        for _ in 0..6 {
            let digit = DigitDisplay::new(
                config.clock.size,
                config.clock.stroke_width,
                config.clock.clock_gap,
                clock_colors.clone(),
                config.clock.animation_duration_ms,
            );
            digits.push(digit);
        }

        // Create separators
        let sep1 = Self::create_separator(separator_color.clone());
        let sep2 = Self::create_separator(separator_color.clone());

        // Build the layout: HH : MM : SS
        let hours_box = GtkBox::new(Orientation::Horizontal, config.clock.digit_gap);
        hours_box.append(digits[0].widget());
        hours_box.append(digits[1].widget());

        let minutes_box = GtkBox::new(Orientation::Horizontal, config.clock.digit_gap);
        minutes_box.append(digits[2].widget());
        minutes_box.append(digits[3].widget());

        let seconds_box = GtkBox::new(Orientation::Horizontal, config.clock.digit_gap);
        seconds_box.append(digits[4].widget());
        seconds_box.append(digits[5].widget());

        container.append(&hours_box);
        container.append(&sep1);
        container.append(&minutes_box);
        container.append(&sep2);
        container.append(&seconds_box);

        ClockDisplay { container, digits }
    }

    /// Creates a separator widget with two dots (`:` character).
    ///
    /// Renders two circular dots vertically aligned to separate digit groups.
    ///
    /// # Arguments
    /// * `color` - Shared reference to the separator color (allows dynamic updates)
    ///
    /// # Returns
    /// A GTK `DrawingArea` widget rendering the separator dots
    fn create_separator(color: Rc<RefCell<(f64, f64, f64, f64)>>) -> DrawingArea {
        let separator = DrawingArea::new();
        separator.set_content_width(SEPARATOR_WIDTH);
        separator.set_content_height(SEPARATOR_HEIGHT);
        separator.set_halign(gtk4::Align::Center);
        separator.set_valign(gtk4::Align::Center);

        separator.set_draw_func(glib::clone!(
            #[strong]
            color,
            move |_, cr, width, _height| {
                let (r, g, b, a) = *color.borrow();

                cr.set_source_rgba(r, g, b, a);

                let center_x = width as f64 / 2.0;

                // Top dot
                cr.arc(
                    center_x,
                    SEPARATOR_TOP_DOT_Y,
                    SEPARATOR_DOT_RADIUS,
                    0.0,
                    2.0 * PI,
                );
                cr.fill().ok();

                // Bottom dot
                cr.arc(
                    center_x,
                    SEPARATOR_BOTTOM_DOT_Y,
                    SEPARATOR_DOT_RADIUS,
                    0.0,
                    2.0 * PI,
                );
                cr.fill().ok();
            }
        ));

        separator
    }

    /// Updates the clock display to show the current system time.
    ///
    /// Reads the system time and animates each digit to match. Called by a
    /// timer every second to keep the display synchronized.
    pub fn update_time(&self) {
        let now = Local::now();
        let time_str = now.format("%H%M%S").to_string();

        for (i, ch) in time_str.chars().enumerate() {
            if let Some(digit_val) = ch.to_digit(10)
                && let Some(digit) = self.digits.get(i)
            {
                digit.set_digit(digit_val as u8);
            }
        }
    }

    /// Updates the clock display immediately without animation.
    ///
    /// Similar to `update_time()` but uses immediate updates instead of animations.
    /// Used after config reload to avoid animating from the old to new display.
    pub fn update_time_immediate(&self) {
        let now = Local::now();
        let time_str = now.format("%H%M%S").to_string();

        for (i, ch) in time_str.chars().enumerate() {
            if let Some(digit_val) = ch.to_digit(10)
                && let Some(digit) = self.digits.get(i)
            {
                digit.set_digit_immediate(digit_val as u8);
            }
        }
    }

    /// Returns a reference to the root widget for this display.
    ///
    /// # Returns
    /// A GTK `Widget` reference that can be added to containers
    pub fn widget(&self) -> &Widget {
        self.container.upcast_ref()
    }
}
