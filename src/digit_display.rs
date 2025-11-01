//! Single digit display using a 6x4 grid of analog clocks.
//!
//! This module creates a display for a single digit (0-9) using 24 analog clocks
//! arranged in a 6-row by 4-column grid. Each clock's hand positions are set
//! according to predefined patterns that visually form the digit shape.
//!
//! # Layout
//! ```text
//! ┌─┬─┬─┬─┐
//! │ │ │ │ │  Row 0
//! ├─┼─┼─┼─┤
//! │ │ │ │ │  Row 1
//! ├─┼─┼─┼─┤
//! │ │ │ │ │  Row 2
//! ├─┼─┼─┼─┤
//! │ │ │ │ │  Row 3
//! ├─┼─┼─┼─┤
//! │ │ │ │ │  Row 4
//! ├─┼─┼─┼─┤
//! │ │ │ │ │  Row 5
//! └─┴─┴─┴─┘
//!   0 1 2 3  (columns)
//! ```

use gtk4::prelude::*;
use gtk4::{Grid, Widget};

use crate::analog_clock::{AnalogClock, ClockColors};
use crate::digit_patterns::get_digit_pattern;

pub struct DigitDisplay {
    container: Grid,
    clocks: Vec<Vec<AnalogClock>>,
}

impl DigitDisplay {
    /// Creates a new digit display with a 6x4 grid of analog clocks.
    ///
    /// Constructs 24 analog clocks arranged in a grid that will be configured
    /// to display digit shapes (0-9).
    ///
    /// # Arguments
    /// * `size` - Size of each individual clock in pixels
    /// * `stroke_width` - Width of clock hands in pixels
    /// * `gap` - Spacing between clocks in pixels
    /// * `colors` - Color scheme for active/inactive clocks
    /// * `animation_duration_ms` - Duration of hand rotation animations
    ///
    /// # Returns
    /// A new `DigitDisplay` ready to display any digit 0-9
    pub fn new(
        size: i32,
        stroke_width: f64,
        gap: i32,
        colors: ClockColors,
        animation_duration_ms: u64,
    ) -> Self {
        let container = Grid::new();
        container.set_row_spacing(gap as u32);
        container.set_column_spacing(gap as u32);

        // Create 6 rows x 4 columns of analog clocks
        let mut clocks = Vec::new();

        for row in 0..6 {
            let mut row_clocks = Vec::new();
            for col in 0..4 {
                let clock =
                    AnalogClock::new(size, stroke_width, colors.clone(), animation_duration_ms);
                container.attach(&clock, col, row, 1, 1);
                row_clocks.push(clock);
            }
            clocks.push(row_clocks);
        }

        DigitDisplay { container, clocks }
    }

    /// Sets this display to show a specific digit with animation.
    ///
    /// Looks up the pattern for the given digit and animates all 24 clocks
    /// to the appropriate hand positions to form the digit shape.
    ///
    /// # Arguments
    /// * `digit` - The digit to display (0-9)
    pub fn set_digit(&self, digit: u8) {
        let pattern = get_digit_pattern(digit);

        for (row_idx, row) in pattern.iter().enumerate() {
            for (col_idx, clock_pos) in row.iter().enumerate() {
                if let Some(clock_row) = self.clocks.get(row_idx)
                    && let Some(clock) = clock_row.get(col_idx)
                {
                    clock.set_angles(clock_pos.hour, clock_pos.minute);
                }
            }
        }
    }

    /// Sets this display to show a specific digit immediately without animation.
    ///
    /// Similar to `set_digit()` but updates instantly. Used after config reload.
    ///
    /// # Arguments
    /// * `digit` - The digit to display (0-9)
    pub fn set_digit_immediate(&self, digit: u8) {
        let pattern = get_digit_pattern(digit);

        for (row_idx, row) in pattern.iter().enumerate() {
            for (col_idx, clock_pos) in row.iter().enumerate() {
                if let Some(clock_row) = self.clocks.get(row_idx)
                    && let Some(clock) = clock_row.get(col_idx)
                {
                    clock.set_angles_immediate(clock_pos.hour, clock_pos.minute);
                }
            }
        }
    }

    /// Returns a reference to the root widget for this digit display.
    ///
    /// # Returns
    /// A GTK `Widget` reference that can be added to containers
    pub fn widget(&self) -> &Widget {
        self.container.upcast_ref()
    }
}
