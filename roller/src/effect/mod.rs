use crate::color::Hsl64;
use palette::Mix;
use serde::{Deserialize, Serialize};

mod color;
mod dimmer;
mod pixel;
mod position;
mod waveform;

pub use color::{ColorEffect, ColorModulation, ColorModulator};
pub use dimmer::{DimmerEffect, DimmerModulator};
pub use pixel::{PixelEffect, PixelModulator, PixelRangeSet};
pub use position::{PositionEffect, PositionModulator};
pub use waveform::Waveform;

use roller_protocol::clock::Beats;

use crate::clock::ClockSnapshot;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EffectDirection {
    BottomToTop,
    ToCenter,
    FromCenter,
    LeftToRight,
}

// Utilities
pub fn intensity(dimmer: f64, intensity: f64) -> f64 {
    1.0 - intensity + dimmer * intensity
}

pub fn color_intensity(color: Hsl64, effected_color: Hsl64, effect_intensity: f64) -> Hsl64 {
    color.mix(&effected_color, effect_intensity)
}

// Takes an intensity between 0.0 - 1.0
// intensities below 0.5 will have more dynamics, above 0.5 will have less
pub fn compress(x: f64, intensity: f64) -> f64 {
    let y = 1.0 / {
        if intensity > 0.5 {
            intensity * 2.0
        } else {
            intensity / 2.0 + 0.5
        }
    };
    f64::powf(x, y)
}

// Adapted from https://math.stackexchange.com/a/3253471
pub fn sigmoid(x: f64, tilt: f64) -> f64 {
    1.0 - (1.0 / (1.0 + f64::powf(1.0 / x - 1.0, -tilt)))
}

pub trait Step {
    fn meter_length(&self) -> Beats;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Steps<T> {
    steps: Vec<T>,
}
impl<T> Steps<T>
where
    T: Step,
{
    fn new(steps: Vec<T>) -> Steps<T> {
        Steps { steps }
    }

    fn total_length(&self) -> Beats {
        self.steps
            .iter()
            .map(|modulator| modulator.meter_length())
            .sum()
    }

    fn current_step<'a>(&'a self, clock: &ClockSnapshot) -> (&'a T, f64) {
        let total_length = self.total_length();
        let elapsed_percent = clock.meter_elapsed_percent(total_length);
        let mut elapsed_beats = total_length * elapsed_percent;

        for step in self.steps.iter() {
            if step.meter_length() >= elapsed_beats {
                return (
                    step,
                    1.0 / f64::from(step.meter_length()) * f64::from(elapsed_beats),
                );
            } else {
                elapsed_beats = elapsed_beats - step.meter_length();
            }
        }

        unreachable!()
    }
}
