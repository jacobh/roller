use palette::{Hsl, Hue, RgbHue};

use crate::color::Hsl64;
use crate::clock::{Beats, ClockSnapshot};

pub struct DimmerEffect {
    effect: Box<dyn Fn(f64) -> f64>,
    meter_beats: Beats,
    intensity: f64,
}
impl DimmerEffect {
    pub fn new(
        effect: impl Fn(f64) -> f64 + 'static,
        meter_beats: Beats,
        intensity: f64,
    ) -> DimmerEffect {
        DimmerEffect {
            meter_beats,
            intensity,
            effect: Box::new(effect),
        }
    }
    pub fn dimmer(&self, clock: &ClockSnapshot) -> f64 {
        let progress = clock.meter_progress(self.meter_beats);
        intensity((self.effect)(progress), self.intensity)
    }
}

// Effects
pub fn saw_up(progress: f64) -> f64 {
    progress
}

pub fn saw_down(progress: f64) -> f64 {
    1.0 - progress
}

pub fn triangle_down(progress: f64) -> f64 {
    if progress > 0.5 {
        (progress - 0.5) * 2.0
    } else {
        1.0 - (progress * 2.0)
    }
}

pub fn sine(progress: f64) -> f64 {
    (f64::sin(std::f64::consts::PI * 2.0 * progress) / 2.0) + 0.5
}

// color effects
pub struct ColorEffect {
    effect: Box<dyn Fn(Hsl64, f64) -> Hsl64>,
    meter_beats: Beats,
}
impl ColorEffect {
    pub fn new(
        effect: impl Fn(Hsl64, f64) -> Hsl64 + 'static,
        meter_beats: Beats,
    ) -> ColorEffect {
        ColorEffect {
            meter_beats,
            effect: Box::new(effect),
        }
    }
    pub fn color(&self, color: Hsl64, clock: &ClockSnapshot) -> Hsl64 {
        let progress = clock.meter_progress(self.meter_beats);
        (self.effect)(color, progress)
    }
}

pub fn hue_shift_30(color: Hsl64, progress: f64) -> Hsl64 {
    color.shift_hue(RgbHue::<f64>::from_degrees(sine(progress) * 30.0))
}

// Utilities
pub fn intensity(dimmer: f64, intensity: f64) -> f64 {
    1.0 - intensity + dimmer * intensity
}
