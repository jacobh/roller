use palette::{Hsl, Hue, Mix, RgbHue};

use crate::clock::{Beats, ClockSnapshot};
use crate::color::Hsl64;

pub struct DimmerEffect {
    effect: Box<dyn Fn(f64) -> f64>,
    meter_length: Beats,
    intensity: f64,
}
impl DimmerEffect {
    pub fn new(
        effect: impl Fn(f64) -> f64 + 'static,
        meter_length: Beats,
        intensity: f64,
    ) -> DimmerEffect {
        DimmerEffect {
            meter_length,
            intensity,
            effect: Box::new(effect),
        }
    }
    pub fn dimmer(&self, clock: &ClockSnapshot) -> f64 {
        let progress_percent = clock.meter_progress_percent(self.meter_length);
        intensity((self.effect)(progress_percent), self.intensity)
    }
}

// Effects for `x` in the range 0.0 - 1.0
pub fn saw_up(x: f64) -> f64 {
    x
}

pub fn saw_down(x: f64) -> f64 {
    1.0 - x
}

pub fn triangle_down(x: f64) -> f64 {
    if x > 0.5 {
        (x - 0.5) * 2.0
    } else {
        1.0 - (x * 2.0)
    }
}

pub fn sine(x: f64) -> f64 {
    (f64::sin(std::f64::consts::PI * 2.0 * x) / 2.0) + 0.5
}

// color effects
pub struct ColorEffect {
    effect: Box<dyn Fn(Hsl64, f64) -> Hsl64>,
    meter_length: Beats,
}
impl ColorEffect {
    pub fn new(effect: impl Fn(Hsl64, f64) -> Hsl64 + 'static, meter_length: Beats) -> ColorEffect {
        ColorEffect {
            meter_length,
            effect: Box::new(effect),
        }
    }
    pub fn color(&self, color: Hsl64, clock: &ClockSnapshot) -> Hsl64 {
        let progress_percent = clock.meter_progress_percent(self.meter_length);
        (self.effect)(color, progress_percent)
    }
}

pub fn hue_shift_30(color: Hsl64, progress_percent: f64) -> Hsl64 {
    color.shift_hue(RgbHue::<f64>::from_degrees(
        triangle_down(progress_percent) * 30.0,
    ))
}

// Utilities
pub fn intensity(dimmer: f64, intensity: f64) -> f64 {
    1.0 - intensity + dimmer * intensity
}

pub fn color_intensity(color: Hsl64, effected_color: Hsl64, effect_intensity: f64) -> Hsl64 {
    color.mix(&effected_color, effect_intensity)
}
