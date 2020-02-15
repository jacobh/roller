use palette::Mix;
use crate::{
    color::{Hsl64},
};

mod color;
mod dimmer;
mod waveform;

pub use waveform::Waveform;
pub use color::{ColorEffect, ColorModulation, ColorModulator};
pub use dimmer::{DimmerEffect, DimmerModulator};

// Utilities
pub fn intensity(dimmer: f64, intensity: f64) -> f64 {
    1.0 - intensity + dimmer * intensity
}

pub fn color_intensity(color: Hsl64, effected_color: Hsl64, effect_intensity: f64) -> Hsl64 {
    color.mix(&effected_color, effect_intensity)
}
