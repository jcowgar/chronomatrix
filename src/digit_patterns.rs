//! Digit shape definitions using analog clock hand positions.
//!
//! This module defines the visual patterns for digits 0-9, where each digit
//! is represented as a 6x4 grid of clock hand positions. Active clocks have
//! their hands positioned to form the digit shape, while inactive clocks
//! use a diagonal "rest" position.
//!
//! # Angle Convention
//! - 0° = 12 o'clock (top)
//! - 90° = 3 o'clock (right)
//! - 180° = 6 o'clock (bottom)
//! - 270° = 9 o'clock (left)
//!
//! # Inactive Position
//! Background clocks that don't contribute to the digit shape use the
//! `INACTIVE` constant (135°, 315°), which creates a diagonal SE-pointing
//! appearance.

/// Represents a single analog clock's hand positions
#[derive(Debug, Clone, Copy)]
pub struct ClockPosition {
    pub hour: i32,   // Hour hand angle in degrees (0-359)
    pub minute: i32, // Minute hand angle in degrees (0-359)
}

impl ClockPosition {
    pub const fn new(hour: i32, minute: i32) -> Self {
        ClockPosition { hour, minute }
    }

    /// Inactive clock (diagonal position)
    pub const INACTIVE: Self = ClockPosition {
        hour: 135,
        minute: 315,
    };
}

/// Each digit is represented as a 6x4 grid (6 rows, 4 columns) of analog clocks
pub type DigitPattern = [[ClockPosition; 4]; 6];

const X: ClockPosition = ClockPosition::INACTIVE;

/// Returns the clock pattern for a given digit.
///
/// Each pattern is a 6x4 array of `ClockPosition` structs defining the
/// hour and minute hand angles for all 24 clocks in the digit grid.
///
/// # Arguments
/// * `digit` - The digit (0-9) to get the pattern for
///
/// # Returns
/// A reference to a static `DigitPattern` array. Invalid digits default to 0.
pub fn get_digit_pattern(digit: u8) -> &'static DigitPattern {
    match digit {
        0 => &DIGIT_0,
        1 => &DIGIT_1,
        2 => &DIGIT_2,
        3 => &DIGIT_3,
        4 => &DIGIT_4,
        5 => &DIGIT_5,
        6 => &DIGIT_6,
        7 => &DIGIT_7,
        8 => &DIGIT_8,
        9 => &DIGIT_9,
        _ => &DIGIT_0,
    }
}

// 0° = 12, 90° = 3, 180° = 6, 270° = 9

const DIGIT_0: DigitPattern = [
    // Row 0
    [
        ClockPosition::new(90, 180),
        ClockPosition::new(90, 270),
        ClockPosition::new(90, 270),
        ClockPosition::new(180, 270),
    ],
    // Row 1
    [
        ClockPosition::new(0, 180),
        ClockPosition::new(90, 180),
        ClockPosition::new(180, 270),
        ClockPosition::new(0, 180),
    ],
    // Row 2
    [
        ClockPosition::new(0, 180),
        ClockPosition::new(0, 180),
        ClockPosition::new(0, 180),
        ClockPosition::new(0, 180),
    ],
    // Row 3
    [
        ClockPosition::new(0, 180),
        ClockPosition::new(0, 180),
        ClockPosition::new(0, 180),
        ClockPosition::new(0, 180),
    ],
    // Row 4
    [
        ClockPosition::new(0, 180),
        ClockPosition::new(0, 90),
        ClockPosition::new(0, 270),
        ClockPosition::new(0, 180),
    ],
    // Row 5
    [
        ClockPosition::new(0, 90),
        ClockPosition::new(90, 270),
        ClockPosition::new(90, 270),
        ClockPosition::new(0, 270),
    ],
];

const DIGIT_1: DigitPattern = [
    // Row 0
    [
        ClockPosition::new(90, 180),
        ClockPosition::new(90, 270),
        ClockPosition::new(270, 180),
        X,
    ],
    // Row 1
    [
        ClockPosition::new(0, 90),
        ClockPosition::new(270, 180),
        ClockPosition::new(0, 180),
        X,
    ],
    // Row 2
    [X, ClockPosition::new(0, 180), ClockPosition::new(0, 180), X],
    // Row 3
    [X, ClockPosition::new(0, 180), ClockPosition::new(0, 180), X],
    // Row 4
    [
        ClockPosition::new(90, 180),
        ClockPosition::new(270, 0),
        ClockPosition::new(0, 90),
        ClockPosition::new(270, 180),
    ],
    // Row 5
    [
        ClockPosition::new(0, 90),
        ClockPosition::new(90, 270),
        ClockPosition::new(90, 270),
        ClockPosition::new(0, 270),
    ],
];

const DIGIT_2: DigitPattern = [
    // Row 0
    [
        ClockPosition::new(90, 180),
        ClockPosition::new(90, 270),
        ClockPosition::new(90, 270),
        ClockPosition::new(180, 270),
    ],
    // Row 1
    [
        ClockPosition::new(0, 90),
        ClockPosition::new(90, 270),
        ClockPosition::new(180, 270),
        ClockPosition::new(0, 180),
    ],
    // Row 2
    [
        ClockPosition::new(90, 180),
        ClockPosition::new(90, 270),
        ClockPosition::new(0, 270),
        ClockPosition::new(0, 180),
    ],
    // Row 3
    [
        ClockPosition::new(0, 180),
        ClockPosition::new(90, 180),
        ClockPosition::new(90, 270),
        ClockPosition::new(0, 270),
    ],
    // Row 4
    [
        ClockPosition::new(0, 180),
        ClockPosition::new(0, 90),
        ClockPosition::new(90, 270),
        ClockPosition::new(180, 270),
    ],
    // Row 5
    [
        ClockPosition::new(0, 90),
        ClockPosition::new(90, 270),
        ClockPosition::new(90, 270),
        ClockPosition::new(0, 270),
    ],
];

const DIGIT_3: DigitPattern = [
    // Row 0
    [
        ClockPosition::new(90, 180),
        ClockPosition::new(90, 270),
        ClockPosition::new(90, 270),
        ClockPosition::new(180, 270),
    ],
    // Row 1
    [
        ClockPosition::new(0, 90),
        ClockPosition::new(90, 270),
        ClockPosition::new(180, 270),
        ClockPosition::new(0, 180),
    ],
    // Row 2
    [
        X,
        ClockPosition::new(90, 180),
        ClockPosition::new(0, 270),
        ClockPosition::new(0, 180),
    ],
    // Row 3
    [
        X,
        ClockPosition::new(0, 90),
        ClockPosition::new(180, 270),
        ClockPosition::new(0, 180),
    ],
    // Row 4
    [
        ClockPosition::new(90, 180),
        ClockPosition::new(90, 270),
        ClockPosition::new(0, 270),
        ClockPosition::new(0, 180),
    ],
    // Row 5
    [
        ClockPosition::new(0, 90),
        ClockPosition::new(90, 270),
        ClockPosition::new(90, 270),
        ClockPosition::new(0, 270),
    ],
];

const DIGIT_4: DigitPattern = [
    // Row 0
    [
        ClockPosition::new(90, 180),
        ClockPosition::new(180, 270),
        ClockPosition::new(180, 90),
        ClockPosition::new(180, 270),
    ],
    // Row 1
    [
        ClockPosition::new(0, 180),
        ClockPosition::new(0, 180),
        ClockPosition::new(0, 180),
        ClockPosition::new(0, 180),
    ],
    // Row 2
    [
        ClockPosition::new(0, 180),
        ClockPosition::new(0, 90),
        ClockPosition::new(0, 270),
        ClockPosition::new(0, 180),
    ],
    // Row 3
    [
        ClockPosition::new(0, 90),
        ClockPosition::new(90, 270),
        ClockPosition::new(180, 270),
        ClockPosition::new(0, 180),
    ],
    // Row 4
    [X, X, ClockPosition::new(0, 180), ClockPosition::new(0, 180)],
    // Row 5
    [X, X, ClockPosition::new(0, 90), ClockPosition::new(0, 270)],
];

const DIGIT_5: DigitPattern = [
    // Row 0
    [
        ClockPosition::new(180, 90),
        ClockPosition::new(270, 90),
        ClockPosition::new(270, 90),
        ClockPosition::new(270, 180),
    ],
    // Row 1
    [
        ClockPosition::new(0, 180),
        ClockPosition::new(180, 90),
        ClockPosition::new(270, 90),
        ClockPosition::new(0, 270),
    ],
    // Row 2
    [
        ClockPosition::new(0, 180),
        ClockPosition::new(0, 90),
        ClockPosition::new(270, 90),
        ClockPosition::new(270, 180),
    ],
    // Row 3
    [
        ClockPosition::new(0, 90),
        ClockPosition::new(270, 90),
        ClockPosition::new(270, 180),
        ClockPosition::new(0, 180),
    ],
    // Row 4
    [
        ClockPosition::new(180, 90),
        ClockPosition::new(270, 90),
        ClockPosition::new(0, 270),
        ClockPosition::new(0, 180),
    ],
    // Row 5
    [
        ClockPosition::new(0, 90),
        ClockPosition::new(270, 90),
        ClockPosition::new(270, 90),
        ClockPosition::new(0, 270),
    ],
];

const DIGIT_6: DigitPattern = [
    // Row 0
    [
        ClockPosition::new(180, 90),
        ClockPosition::new(180, 270),
        X,
        X,
    ],
    // Row 1
    [ClockPosition::new(180, 0), ClockPosition::new(180, 0), X, X],
    // Row 2
    [
        ClockPosition::new(180, 0),
        ClockPosition::new(0, 90),
        ClockPosition::new(270, 90),
        ClockPosition::new(180, 270),
    ],
    // Row 3
    [
        ClockPosition::new(180, 0),
        ClockPosition::new(180, 90),
        ClockPosition::new(180, 270),
        ClockPosition::new(180, 0),
    ],
    // Row 4
    [
        ClockPosition::new(180, 0),
        ClockPosition::new(0, 90),
        ClockPosition::new(270, 0),
        ClockPosition::new(180, 0),
    ],
    // Row 5
    [
        ClockPosition::new(0, 90),
        ClockPosition::new(270, 90),
        ClockPosition::new(270, 90),
        ClockPosition::new(270, 0),
    ],
];

const DIGIT_7: DigitPattern = [
    // Row 0
    [
        ClockPosition::new(90, 180),
        ClockPosition::new(90, 270),
        ClockPosition::new(90, 270),
        ClockPosition::new(180, 270),
    ],
    // Row 1
    [
        ClockPosition::new(0, 90),
        ClockPosition::new(90, 270),
        ClockPosition::new(180, 270),
        ClockPosition::new(0, 180),
    ],
    // Row 2
    [X, X, ClockPosition::new(0, 180), ClockPosition::new(0, 180)],
    // Row 3
    [X, X, ClockPosition::new(0, 180), ClockPosition::new(0, 180)],
    // Row 4
    [X, X, ClockPosition::new(0, 180), ClockPosition::new(0, 180)],
    // Row 5
    [X, X, ClockPosition::new(0, 90), ClockPosition::new(0, 270)],
];

const DIGIT_8: DigitPattern = [
    // Row 0
    [
        ClockPosition::new(90, 180),
        ClockPosition::new(90, 270),
        ClockPosition::new(90, 270),
        ClockPosition::new(180, 270),
    ],
    // Row 1
    [
        ClockPosition::new(0, 180),
        ClockPosition::new(90, 180),
        ClockPosition::new(180, 270),
        ClockPosition::new(0, 180),
    ],
    // Row 2
    [
        ClockPosition::new(0, 180),
        ClockPosition::new(0, 90),
        ClockPosition::new(0, 270),
        ClockPosition::new(0, 180),
    ],
    // Row 3
    [
        ClockPosition::new(0, 180),
        ClockPosition::new(90, 180),
        ClockPosition::new(180, 270),
        ClockPosition::new(0, 180),
    ],
    // Row 4
    [
        ClockPosition::new(0, 180),
        ClockPosition::new(0, 90),
        ClockPosition::new(0, 270),
        ClockPosition::new(0, 180),
    ],
    // Row 5
    [
        ClockPosition::new(0, 90),
        ClockPosition::new(90, 270),
        ClockPosition::new(90, 270),
        ClockPosition::new(0, 270),
    ],
];

const DIGIT_9: DigitPattern = [
    // Row 0
    [
        ClockPosition::new(90, 180),
        ClockPosition::new(90, 270),
        ClockPosition::new(90, 270),
        ClockPosition::new(180, 270),
    ],
    // Row 1
    [
        ClockPosition::new(0, 180),
        ClockPosition::new(90, 180),
        ClockPosition::new(180, 270),
        ClockPosition::new(0, 180),
    ],
    // Row 2
    [
        ClockPosition::new(0, 180),
        ClockPosition::new(0, 90),
        ClockPosition::new(0, 270),
        ClockPosition::new(0, 180),
    ],
    // Row 3
    [
        ClockPosition::new(0, 90),
        ClockPosition::new(90, 270),
        ClockPosition::new(180, 270),
        ClockPosition::new(0, 180),
    ],
    // Row 4
    [X, X, ClockPosition::new(0, 180), ClockPosition::new(0, 180)],
    // Row 5
    [X, X, ClockPosition::new(0, 90), ClockPosition::new(0, 270)],
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clock_position_inactive() {
        let inactive = ClockPosition::INACTIVE;
        assert_eq!(inactive.hour, 135);
        assert_eq!(inactive.minute, 315);
    }

    #[test]
    fn test_get_digit_pattern_all_digits() {
        // Test that all digits 0-9 return valid patterns
        for digit in 0..=9 {
            let pattern = get_digit_pattern(digit);
            // Each pattern should be 6 rows
            assert_eq!(pattern.len(), 6);
            // Each row should have 4 columns
            for row in pattern.iter() {
                assert_eq!(row.len(), 4);
            }
        }
    }

    #[test]
    fn test_get_digit_pattern_invalid_digit() {
        // Invalid digit should default to 0
        let pattern = get_digit_pattern(99);
        let zero_pattern = get_digit_pattern(0);

        // Both should point to the same pattern
        assert_eq!(pattern.len(), zero_pattern.len());
        assert_eq!(pattern[0][0].hour, zero_pattern[0][0].hour);
    }

    #[test]
    fn test_digit_pattern_angles_valid() {
        // Verify that all angles in all patterns are within valid range
        for digit in 0..=9 {
            let pattern = get_digit_pattern(digit);
            for row in pattern.iter() {
                for clock_pos in row.iter() {
                    assert!(
                        clock_pos.hour >= 0 && clock_pos.hour < 360,
                        "Invalid hour angle {} in digit {}",
                        clock_pos.hour,
                        digit
                    );
                    assert!(
                        clock_pos.minute >= 0 && clock_pos.minute < 360,
                        "Invalid minute angle {} in digit {}",
                        clock_pos.minute,
                        digit
                    );
                }
            }
        }
    }
}
