use crate::color::Hsl64;
use palette::Mix;

mod color;
mod dimmer;
mod waveform;

pub use color::{ColorEffect, ColorModulation, ColorModulator};
pub use dimmer::{DimmerEffect, DimmerModulator};
pub use waveform::Waveform;

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
