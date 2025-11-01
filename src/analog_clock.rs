//! Individual analog clock widget implementation.
//!
//! This module provides a custom GTK4 widget that renders a single analog clock
//! using Cairo. Each clock displays two hands (hour and minute) that can be
//! positioned at any angle.
//!
//! # Features
//! - **Smooth animations**: 60 FPS updates with ease-in-out easing
//! - **Cumulative angle tracking**: Ensures hands always rotate clockwise, never backwards
//! - **Active/inactive states**: Different colors for clocks forming digits vs background clocks
//! - **Color transitions**: Smooth color interpolation during state changes
//!
//! # Animation System
//! When `set_angles()` is called, the clock:
//! 1. Calculates cumulative target angles (preventing backwards rotation)
//! 2. Starts a timer that fires every 16ms (~60 FPS)
//! 3. Interpolates between start and target angles using ease-in-out curve
//! 4. Updates the display and transitions colors smoothly
//!
//! # Angle Convention
//! - Angles are in degrees (0-359)
//! - 0° points to 12 o'clock (top)
//! - 90° points to 3 o'clock (right)
//! - 180° points to 6 o'clock (bottom)
//! - 270° points to 9 o'clock (left)

use cairo::Context;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use gtk4::{DrawingArea, glib};
use std::cell::RefCell;
use std::f64::consts::PI;
use std::time::Duration;

/// Inactive clock position - hour hand at 135° (SE diagonal)
const INACTIVE_HOUR_ANGLE: i32 = 135;

/// Inactive clock position - minute hand at 315° (SE diagonal)
const INACTIVE_MINUTE_ANGLE: i32 = 315;

/// Alternative inactive position - hour hand at 225° (SW diagonal)
const ALT_INACTIVE_HOUR_ANGLE: i32 = 225;

/// Alternative inactive position - minute hand at 225° (SW diagonal)
const ALT_INACTIVE_MINUTE_ANGLE: i32 = 225;

/// Animation frame rate in frames per second
const ANIMATION_FPS: u64 = 60;

/// Animation frame duration in milliseconds (16ms = ~60 FPS)
const FRAME_DURATION_MS: u64 = 1000 / ANIMATION_FPS;

/// Angle offset to make 0° point to 12 o'clock instead of 3 o'clock
const ANGLE_OFFSET_DEGREES: f64 = -90.0;

/// Clock face border width in pixels
const CLOCK_BORDER_WIDTH: f64 = 1.0;

/// Radius reduction for clock face (creates padding inside border)
const CLOCK_RADIUS_PADDING: f64 = 2.0;

/// Hand length reduction from clock radius (creates gap at edge)
const HAND_LENGTH_REDUCTION: f64 = 5.0;

/// Center dot opacity when clock is active
const CENTER_DOT_OPACITY_ACTIVE: f64 = 0.5;

/// Center dot opacity when clock is inactive
const CENTER_DOT_OPACITY_INACTIVE: f64 = 1.0;

#[derive(Debug, Clone)]
pub struct ClockColors {
    pub active_color: (f64, f64, f64, f64),
    pub inactive_color: (f64, f64, f64, f64),
    pub bg_color: (f64, f64, f64, f64),
    pub border_color: (f64, f64, f64, f64),
}

mod imp {
    use super::*;

    #[derive(Debug)]
    pub struct AnalogClock {
        pub hour_angle: RefCell<f64>,
        pub minute_angle: RefCell<f64>,
        pub cumulative_hour_angle: RefCell<f64>,
        pub cumulative_minute_angle: RefCell<f64>,
        pub target_cumulative_hour: RefCell<f64>,
        pub target_cumulative_minute: RefCell<f64>,
        pub last_hour_angle: RefCell<Option<f64>>,
        pub last_minute_angle: RefCell<Option<f64>>,
        pub size: RefCell<i32>,
        pub stroke_width: RefCell<f64>,
        pub colors: RefCell<ClockColors>,
        pub is_active: RefCell<bool>,
        pub target_is_active: RefCell<bool>,
        pub animation_duration_ms: RefCell<u64>,
        pub animation_start_time: RefCell<Option<std::time::Instant>>,
        pub start_cumulative_hour: RefCell<f64>,
        pub start_cumulative_minute: RefCell<f64>,
    }

    impl Default for AnalogClock {
        fn default() -> Self {
            Self {
                hour_angle: RefCell::new(0.0),
                minute_angle: RefCell::new(0.0),
                cumulative_hour_angle: RefCell::new(0.0),
                cumulative_minute_angle: RefCell::new(0.0),
                target_cumulative_hour: RefCell::new(0.0),
                target_cumulative_minute: RefCell::new(0.0),
                last_hour_angle: RefCell::new(None),
                last_minute_angle: RefCell::new(None),
                size: RefCell::new(40),
                stroke_width: RefCell::new(2.0),
                colors: RefCell::new(ClockColors {
                    active_color: (1.0, 0.42, 0.42, 1.0),    // #ff6b6b
                    inactive_color: (1.0, 0.42, 0.42, 0.15), // #ff6b6b26
                    bg_color: (1.0, 1.0, 1.0, 0.03),         // #ffffff08
                    border_color: (1.0, 1.0, 1.0, 0.1),      // #ffffff1a
                }),
                is_active: RefCell::new(true),
                target_is_active: RefCell::new(true),
                animation_duration_ms: RefCell::new(300),
                animation_start_time: RefCell::new(None),
                start_cumulative_hour: RefCell::new(0.0),
                start_cumulative_minute: RefCell::new(0.0),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AnalogClock {
        const NAME: &'static str = "AnalogClock";
        type Type = super::AnalogClock;
        type ParentType = DrawingArea;
    }

    impl ObjectImpl for AnalogClock {}
    impl WidgetImpl for AnalogClock {}
    impl DrawingAreaImpl for AnalogClock {}
}

glib::wrapper! {
    pub struct AnalogClock(ObjectSubclass<imp::AnalogClock>)
        @extends DrawingArea, gtk4::Widget;
}

impl AnalogClock {
    /// Creates a new analog clock widget.
    ///
    /// Initializes a custom GTK DrawingArea that renders an analog clock face
    /// with two hands. The clock automatically starts its 60 FPS animation loop.
    ///
    /// # Arguments
    /// * `size` - Clock diameter in pixels
    /// * `stroke_width` - Width of clock hands in pixels
    /// * `colors` - Color scheme for active/inactive states and clock face
    /// * `animation_duration_ms` - Duration of hand rotation animations in milliseconds
    ///
    /// # Returns
    /// A new `AnalogClock` widget ready to be added to a GTK container
    pub fn new(
        size: i32,
        stroke_width: f64,
        colors: ClockColors,
        animation_duration_ms: u64,
    ) -> Self {
        let obj: Self = glib::Object::new();

        let imp = obj.imp();
        *imp.size.borrow_mut() = size;
        *imp.stroke_width.borrow_mut() = stroke_width;
        *imp.colors.borrow_mut() = colors;
        *imp.animation_duration_ms.borrow_mut() = animation_duration_ms;

        obj.set_content_width(size);
        obj.set_content_height(size);

        obj.set_draw_func(glib::clone!(
            #[weak]
            obj,
            move |_, cr, width, height| {
                obj.draw(cr, width, height);
            }
        ));

        // Start animation loop
        obj.start_animation_loop();

        obj
    }

    /// Sets the target angles for the clock hands with smooth animation.
    ///
    /// Calculates cumulative target angles to ensure clockwise rotation,
    /// then starts an animated transition from the current angles to the targets.
    /// Also updates the active/inactive state based on whether the clock is at
    /// a "rest" position (diagonal).
    ///
    /// # Arguments
    /// * `hour` - Target hour hand angle in degrees (0-359)
    /// * `minute` - Target minute hand angle in degrees (0-359)
    ///
    /// # Notes
    /// - Hands always rotate clockwise, never backwards
    /// - Animation duration is determined by `animation_duration_ms` from constructor
    /// - Colors transition smoothly between active/inactive states
    pub fn set_angles(&self, hour: i32, minute: i32) {
        let imp = self.imp();

        // Calculate target cumulative angles for smooth clockwise rotation
        let target_hour = self.calculate_target_cumulative(hour as f64, true);
        let target_minute = self.calculate_target_cumulative(minute as f64, false);

        // Store current cumulative as start point for animation
        *imp.start_cumulative_hour.borrow_mut() = *imp.cumulative_hour_angle.borrow();
        *imp.start_cumulative_minute.borrow_mut() = *imp.cumulative_minute_angle.borrow();

        // Set target angles
        *imp.target_cumulative_hour.borrow_mut() = target_hour;
        *imp.target_cumulative_minute.borrow_mut() = target_minute;

        // Start animation
        *imp.animation_start_time.borrow_mut() = Some(std::time::Instant::now());

        *imp.hour_angle.borrow_mut() = hour as f64;
        *imp.minute_angle.borrow_mut() = minute as f64;

        // Check if clock should be active (not at one of the inactive positions)
        let target_active = !(hour == INACTIVE_HOUR_ANGLE && minute == INACTIVE_MINUTE_ANGLE
            || hour == ALT_INACTIVE_HOUR_ANGLE && minute == ALT_INACTIVE_MINUTE_ANGLE);
        *imp.target_is_active.borrow_mut() = target_active;
    }

    /// Sets the clock hand angles immediately without animation.
    ///
    /// Directly updates the clock hands to the specified angles without any
    /// transition. Useful for initialization or when instant updates are needed
    /// (e.g., after config reload).
    ///
    /// # Arguments
    /// * `hour` - Hour hand angle in degrees (0-359)
    /// * `minute` - Minute hand angle in degrees (0-359)
    pub fn set_angles_immediate(&self, hour: i32, minute: i32) {
        let imp = self.imp();

        // Set angles immediately without animation
        let hour_f64 = hour as f64;
        let minute_f64 = minute as f64;

        *imp.hour_angle.borrow_mut() = hour_f64;
        *imp.minute_angle.borrow_mut() = minute_f64;
        *imp.cumulative_hour_angle.borrow_mut() = hour_f64;
        *imp.cumulative_minute_angle.borrow_mut() = minute_f64;
        *imp.target_cumulative_hour.borrow_mut() = hour_f64;
        *imp.target_cumulative_minute.borrow_mut() = minute_f64;
        *imp.last_hour_angle.borrow_mut() = Some(hour_f64);
        *imp.last_minute_angle.borrow_mut() = Some(minute_f64);
        *imp.animation_start_time.borrow_mut() = None;

        // Check if clock is active (not at one of the inactive positions)
        let is_active = !(hour == INACTIVE_HOUR_ANGLE && minute == INACTIVE_MINUTE_ANGLE
            || hour == ALT_INACTIVE_HOUR_ANGLE && minute == ALT_INACTIVE_MINUTE_ANGLE);
        *imp.is_active.borrow_mut() = is_active;
        *imp.target_is_active.borrow_mut() = is_active;

        self.queue_draw();
    }

    /// Calculates the cumulative target angle to ensure clockwise rotation.
    ///
    /// This is the key function that prevents backwards rotation. It compares
    /// the new angle with the last angle and calculates the shortest clockwise
    /// path, adding it to the cumulative angle.
    ///
    /// # How it works
    /// - Normalizes both angles to 0-360°
    /// - Calculates the difference
    /// - If negative, adds 360° to make it positive (clockwise)
    /// - Adds the difference to the cumulative angle
    ///
    /// # Example
    /// If current angle is 350° and new angle is 10°:
    /// - Difference: 10 - 350 = -340°
    /// - Add 360°: -340 + 360 = 20°
    /// - Result: cumulative + 20° (rotates 20° clockwise, not 340° backwards)
    ///
    /// # Arguments
    /// * `new_angle` - The target angle in degrees
    /// * `is_hour` - Whether this is the hour hand (true) or minute hand (false)
    ///
    /// # Returns
    /// The new cumulative angle that ensures clockwise rotation
    fn calculate_target_cumulative(&self, new_angle: f64, is_hour: bool) -> f64 {
        let imp = self.imp();

        if is_hour {
            let last_angle = *imp.last_hour_angle.borrow();
            let current_cumulative = *imp.cumulative_hour_angle.borrow();

            if let Some(last) = last_angle {
                let normalized_last = last % 360.0;
                let normalized_new = new_angle % 360.0;
                let mut diff = normalized_new - normalized_last;

                if diff < 0.0 {
                    diff += 360.0;
                }

                *imp.last_hour_angle.borrow_mut() = Some(normalized_new);
                current_cumulative + diff
            } else {
                *imp.last_hour_angle.borrow_mut() = Some(new_angle);
                new_angle
            }
        } else {
            let last_angle = *imp.last_minute_angle.borrow();
            let current_cumulative = *imp.cumulative_minute_angle.borrow();

            if let Some(last) = last_angle {
                let normalized_last = last % 360.0;
                let normalized_new = new_angle % 360.0;
                let mut diff = normalized_new - normalized_last;

                if diff < 0.0 {
                    diff += 360.0;
                }

                *imp.last_minute_angle.borrow_mut() = Some(normalized_new);
                current_cumulative + diff
            } else {
                *imp.last_minute_angle.borrow_mut() = Some(new_angle);
                new_angle
            }
        }
    }

    /// Starts the 60 FPS animation loop for this clock.
    ///
    /// Creates a timer that fires every ~16ms (60 FPS) to update the animation.
    /// The loop continues for the lifetime of the widget.
    fn start_animation_loop(&self) {
        glib::timeout_add_local(
            Duration::from_millis(FRAME_DURATION_MS),
            glib::clone!(
                #[weak(rename_to = clock)]
                self,
                #[upgrade_or]
                glib::ControlFlow::Break,
                move || {
                    clock.update_animation();
                    glib::ControlFlow::Continue
                }
            ),
        );
    }

    /// Updates the animation state for the current frame.
    ///
    /// Called every 16ms by the animation loop. Interpolates between start
    /// and target angles using an ease-in-out curve. When animation completes,
    /// snaps to exact target values and clears the animation timer.
    fn update_animation(&self) {
        let imp = self.imp();
        let animation_start = *imp.animation_start_time.borrow();

        if let Some(start_time) = animation_start {
            let elapsed = start_time.elapsed().as_millis() as f64;
            let duration = *imp.animation_duration_ms.borrow() as f64;

            if elapsed < duration {
                // Calculate easing (ease-in-out)
                let progress = elapsed / duration;
                let eased_progress = Self::ease_in_out(progress);

                // Interpolate between start and target angles
                let start_hour = *imp.start_cumulative_hour.borrow();
                let target_hour = *imp.target_cumulative_hour.borrow();
                let start_minute = *imp.start_cumulative_minute.borrow();
                let target_minute = *imp.target_cumulative_minute.borrow();

                *imp.cumulative_hour_angle.borrow_mut() =
                    start_hour + (target_hour - start_hour) * eased_progress;
                *imp.cumulative_minute_angle.borrow_mut() =
                    start_minute + (target_minute - start_minute) * eased_progress;

                // Don't immediately update is_active during animation
                // We'll blend colors in the draw function based on progress

                self.queue_draw();
            } else {
                // Animation complete - set to exact target values
                *imp.cumulative_hour_angle.borrow_mut() = *imp.target_cumulative_hour.borrow();
                *imp.cumulative_minute_angle.borrow_mut() = *imp.target_cumulative_minute.borrow();
                *imp.is_active.borrow_mut() = *imp.target_is_active.borrow();
                *imp.animation_start_time.borrow_mut() = None;
                self.queue_draw();
            }
        }
    }

    /// Applies ease-in-out easing to animation progress.
    ///
    /// Creates smooth acceleration at the start and deceleration at the end
    /// of animations using a quadratic easing function.
    ///
    /// # Arguments
    /// * `t` - Linear progress value (0.0 to 1.0)
    ///
    /// # Returns
    /// Eased progress value (0.0 to 1.0) with smooth acceleration/deceleration
    fn ease_in_out(t: f64) -> f64 {
        if t < 0.5 {
            2.0 * t * t
        } else {
            -1.0 + (4.0 - 2.0 * t) * t
        }
    }

    /// Renders the clock to the Cairo context.
    ///
    /// Draws all elements of the clock in order:
    /// 1. Background circle (filled)
    /// 2. Border circle (stroked)
    /// 3. Hour hand
    /// 4. Minute hand
    /// 5. Center dot
    ///
    /// Colors are interpolated during active/inactive transitions for smooth
    /// visual effects.
    ///
    /// # Arguments
    /// * `cr` - Cairo rendering context
    /// * `width` - Widget width in pixels
    /// * `height` - Widget height in pixels
    fn draw(&self, cr: &Context, width: i32, height: i32) {
        let imp = self.imp();

        let size = *imp.size.borrow();
        let stroke_width = *imp.stroke_width.borrow();
        let colors = imp.colors.borrow();
        let is_active = *imp.is_active.borrow();

        let center_x = width as f64 / 2.0;
        let center_y = height as f64 / 2.0;
        let radius = (size as f64 / 2.0) - CLOCK_RADIUS_PADDING;

        // Draw clock background
        let bg = colors.bg_color;
        cr.set_source_rgba(bg.0, bg.1, bg.2, bg.3);
        cr.arc(center_x, center_y, radius, 0.0, 2.0 * PI);
        cr.fill().ok();

        // Draw clock border
        let border = colors.border_color;
        cr.set_source_rgba(border.0, border.1, border.2, border.3);
        cr.set_line_width(CLOCK_BORDER_WIDTH);
        cr.arc(center_x, center_y, radius, 0.0, 2.0 * PI);
        cr.stroke().ok();

        // Interpolate color during animation
        let hand_color = if let Some(start_time) = *imp.animation_start_time.borrow() {
            let elapsed = start_time.elapsed().as_millis() as f64;
            let duration = *imp.animation_duration_ms.borrow() as f64;
            let target_is_active = *imp.target_is_active.borrow();

            // If we're transitioning between states, interpolate colors
            if is_active != target_is_active {
                // Calculate progress (clamped to 1.0)
                let progress = (elapsed / duration).min(1.0);
                let eased_progress = Self::ease_in_out(progress);

                // Interpolate between inactive and active colors
                let (from_color, to_color) = if target_is_active {
                    (colors.inactive_color, colors.active_color)
                } else {
                    (colors.active_color, colors.inactive_color)
                };

                (
                    from_color.0 + (to_color.0 - from_color.0) * eased_progress,
                    from_color.1 + (to_color.1 - from_color.1) * eased_progress,
                    from_color.2 + (to_color.2 - from_color.2) * eased_progress,
                    from_color.3 + (to_color.3 - from_color.3) * eased_progress,
                )
            } else {
                // Same state, use appropriate color
                if is_active {
                    colors.active_color
                } else {
                    colors.inactive_color
                }
            }
        } else {
            // Not animating, use current state
            if is_active {
                colors.active_color
            } else {
                colors.inactive_color
            }
        };

        cr.set_source_rgba(hand_color.0, hand_color.1, hand_color.2, hand_color.3);
        cr.set_line_width(stroke_width);
        cr.set_line_cap(cairo::LineCap::Round);

        // Draw hour hand (using cumulative angle for smooth rotation)
        let cumulative_hour = *imp.cumulative_hour_angle.borrow();
        let hour_rad = (cumulative_hour + ANGLE_OFFSET_DEGREES) * PI / 180.0;
        let hour_length = radius - HAND_LENGTH_REDUCTION;

        cr.move_to(center_x, center_y);
        cr.line_to(
            center_x + hour_length * hour_rad.cos(),
            center_y + hour_length * hour_rad.sin(),
        );
        cr.stroke().ok();

        // Draw minute hand (using cumulative angle for smooth rotation)
        let cumulative_minute = *imp.cumulative_minute_angle.borrow();
        let minute_rad = (cumulative_minute + ANGLE_OFFSET_DEGREES) * PI / 180.0;
        let minute_length = radius - HAND_LENGTH_REDUCTION;

        cr.move_to(center_x, center_y);
        cr.line_to(
            center_x + minute_length * minute_rad.cos(),
            center_y + minute_length * minute_rad.sin(),
        );
        cr.stroke().ok();

        // Draw center dot with animated opacity
        let center_opacity = if let Some(start_time) = *imp.animation_start_time.borrow() {
            let elapsed = start_time.elapsed().as_millis() as f64;
            let duration = *imp.animation_duration_ms.borrow() as f64;
            let target_is_active = *imp.target_is_active.borrow();

            // If we're transitioning between states, interpolate center dot opacity
            if is_active != target_is_active {
                let progress = (elapsed / duration).min(1.0);
                let eased_progress = Self::ease_in_out(progress);

                // Interpolate between inactive and active opacity
                let (from_opacity, to_opacity) = if target_is_active {
                    (CENTER_DOT_OPACITY_INACTIVE, CENTER_DOT_OPACITY_ACTIVE)
                } else {
                    (CENTER_DOT_OPACITY_ACTIVE, CENTER_DOT_OPACITY_INACTIVE)
                };

                from_opacity + (to_opacity - from_opacity) * eased_progress
            } else if is_active {
                CENTER_DOT_OPACITY_ACTIVE
            } else {
                CENTER_DOT_OPACITY_INACTIVE
            }
        } else if is_active {
            CENTER_DOT_OPACITY_ACTIVE
        } else {
            CENTER_DOT_OPACITY_INACTIVE
        };

        cr.set_source_rgba(hand_color.0, hand_color.1, hand_color.2, center_opacity);
        cr.arc(center_x, center_y, stroke_width, 0.0, 2.0 * PI);
        cr.fill().ok();
    }
}
