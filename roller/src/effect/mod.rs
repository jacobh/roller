use ordered_float::OrderedFloat;
use palette::{Hue, Mix, RgbHue};
use std::f64::consts::PI;

use crate::{
    clock::{Beats, ClockOffset, ClockSnapshot},
    color::{Color, Hsl64},
    fixture::Fixture,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DimmerEffect {
    steps: Vec<DimmerModulator>,
    clock_offset: Option<ClockOffset>,
}
impl DimmerEffect {
    pub fn new(steps: Vec<DimmerModulator>, clock_offset: Option<ClockOffset>) -> DimmerEffect {
        DimmerEffect {
            steps,
            clock_offset,
        }
    }
    fn total_length(&self) -> Beats {
        self.steps
            .iter()
            .map(|modulator| modulator.meter_length)
            .sum()
    }
    pub fn dimmer(&self, clock: &ClockSnapshot) -> f64 {
        let length = self.total_length();
        let elapsed_percent = clock.meter_elapsed_percent(length);
        let mut elapsed_beats = length * elapsed_percent;

        for step in self.steps.iter() {
            if step.meter_length >= elapsed_beats {
                return step.dimmer_for_elapsed_percent(
                    1.0 / f64::from(step.meter_length) * f64::from(elapsed_beats),
                );
            } else {
                elapsed_beats = elapsed_beats - step.meter_length;
            }
        }

        unreachable!()
    }
    pub fn offset_dimmer(
        &self,
        clock: &ClockSnapshot,
        fixture: &Fixture,
        fixtures: &[Fixture],
    ) -> f64 {
        match &self.clock_offset {
            Some(clock_offset) => {
                let offset = clock_offset.offset_for_fixture(fixture, fixtures);
                self.dimmer(&clock.shift(offset))
            }
            None => self.dimmer(clock),
        }
    }
}

impl From<DimmerModulator> for DimmerEffect {
    fn from(modulator: DimmerModulator) -> DimmerEffect {
        DimmerEffect::new(vec![modulator], None)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DimmerScale {
    min: OrderedFloat<f64>,
    max: OrderedFloat<f64>,
}
impl DimmerScale {
    pub fn new(min: f64, max: f64) -> DimmerScale {
        DimmerScale {
            min: OrderedFloat::from(min),
            max: OrderedFloat::from(max),
        }
    }
    pub fn scale(&self, x: f64) -> f64 {
        let min = self.min.into_inner();
        let max = self.max.into_inner();

        min + x * (max - min)
    }
}
impl From<(f64, f64)> for DimmerScale {
    fn from(x: (f64, f64)) -> DimmerScale {
        DimmerScale::new(x.0, x.1)
    }
}
impl From<f64> for DimmerScale {
    fn from(x: f64) -> DimmerScale {
        DimmerScale::new(1.0 - x, x)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DimmerModulator {
    waveform: Waveform,
    meter_length: Beats,
    scale: DimmerScale,
}
impl DimmerModulator {
    pub fn new(
        waveform: Waveform,
        meter_length: Beats,
        scale: impl Into<DimmerScale>,
    ) -> DimmerModulator {
        DimmerModulator {
            meter_length,
            waveform,
            scale: scale.into(),
        }
    }
    fn dimmer_for_elapsed_percent(&self, elapsed_percent: f64) -> f64 {
        self.scale.scale(self.waveform.apply(elapsed_percent))
    }
    pub fn dimmer(&self, clock: &ClockSnapshot) -> f64 {
        let elapsed_percent = clock.meter_elapsed_percent(self.meter_length);
        self.dimmer_for_elapsed_percent(elapsed_percent)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Waveform {
    SawUp,
    SawDown,
    TriangleDown,
    SineUp,
    SineDown,
    HalfSineUp,
    HalfSineDown,
    ShortSquarePulse,
    On,
    Off,
}
impl Waveform {
    fn apply(self, x: f64) -> f64 {
        match self {
            Waveform::SawUp => saw_up(x),
            Waveform::SawDown => saw_down(x),
            Waveform::TriangleDown => triangle_down(x),
            Waveform::SineUp => sine_up(x),
            Waveform::SineDown => sine_down(x),
            Waveform::HalfSineUp => half_sine_up(x),
            Waveform::HalfSineDown => half_sine_down(x),
            Waveform::ShortSquarePulse => short_square_pulse(x),
            Waveform::On => 1.0,
            Waveform::Off => 0.0,
        }
    }
}

// Waveforms for `x` in the range 0.0 - 1.0
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

/// 0.0 = 0.0
/// 0.5 = 1.0
/// 1.0 = 0.0
pub fn sine_up(x: f64) -> f64 {
    (f64::sin(PI * 2.0 * x - 1.5) / 2.0) + 0.5
}

/// 0.0 = 1.0
/// 0.5 = 0.0
/// 1.0 = 1.0
pub fn sine_down(x: f64) -> f64 {
    (f64::sin(PI * 2.0 * x + 1.5) / 2.0) + 0.5
}

pub fn half_sine_up(x: f64) -> f64 {
    (f64::sin(((PI * 2.0 * x) / 2.0) - 1.5) / 2.0) + 0.5
}

pub fn half_sine_down(x: f64) -> f64 {
    (f64::sin(((PI * 2.0 * x) / 2.0) + 1.5) / 2.0) + 0.5
}

pub fn short_square_pulse(x: f64) -> f64 {
    if x < 0.2 {
        1.0
    } else {
        f64::max(0.5 - (x / 1.2), 0.0)
    }
}

// color effects
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ColorEffect {
    steps: Vec<ColorModulator>,
    clock_offset: Option<ClockOffset>,
}
impl ColorEffect {
    pub fn new(steps: Vec<ColorModulator>, clock_offset: Option<ClockOffset>) -> ColorEffect {
        ColorEffect {
            steps,
            clock_offset,
        }
    }
    fn total_length(&self) -> Beats {
        self.steps
            .iter()
            .map(|modulator| modulator.meter_length)
            .sum()
    }
    pub fn color(&self, color: Hsl64, clock: &ClockSnapshot) -> Hsl64 {
        let length = self.total_length();
        let elapsed_percent = clock.meter_elapsed_percent(length);
        let mut elapsed_beats = length * elapsed_percent;

        for step in self.steps.iter() {
            if step.meter_length >= elapsed_beats {
                return step.color_for_elapsed_percent(
                    color,
                    1.0 / f64::from(step.meter_length) * f64::from(elapsed_beats),
                );
            } else {
                elapsed_beats = elapsed_beats - step.meter_length;
            }
        }

        unreachable!()
    }

    pub fn offset_color(
        &self,
        color: Hsl64,
        clock: &ClockSnapshot,
        fixture: &Fixture,
        fixtures: &[Fixture],
    ) -> Hsl64 {
        match &self.clock_offset {
            Some(clock_offset) => self.color(
                color,
                &clock.shift(clock_offset.offset_for_fixture(fixture, fixtures)),
            ),
            None => self.color(color, clock),
        }
    }
}

impl From<ColorModulator> for ColorEffect {
    fn from(modulator: ColorModulator) -> ColorEffect {
        ColorEffect::new(vec![modulator], None)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ColorModulation {
    HueShift(OrderedFloat<f64>),
    White,
    NoOp,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ColorModulator {
    modulation: ColorModulation,
    waveform: Waveform,
    meter_length: Beats,
}
impl ColorModulator {
    pub fn new(
        modulation: ColorModulation,
        waveform: Waveform,
        meter_length: Beats,
    ) -> ColorModulator {
        ColorModulator {
            modulation,
            waveform,
            meter_length,
        }
    }
    pub fn new_static(modulation: ColorModulation, meter_length: Beats) -> ColorModulator {
        ColorModulator {
            modulation,
            meter_length,
            waveform: Waveform::On,
        }
    }
    fn color_for_elapsed_percent(&self, color: Hsl64, elapsed_percent: f64) -> Hsl64 {
        match self.modulation {
            ColorModulation::HueShift(shift_degrees) => {
                color.shift_hue(RgbHue::<f64>::from_degrees(
                    self.waveform.apply(elapsed_percent) * shift_degrees.into_inner(),
                ))
            }
            ColorModulation::White => {
                color.mix(&Color::White.to_hsl(), self.waveform.apply(elapsed_percent))
            }
            ColorModulation::NoOp => color,
        }
    }
    pub fn color(&self, color: Hsl64, clock: &ClockSnapshot) -> Hsl64 {
        let elapsed_percent = clock.meter_elapsed_percent(self.meter_length);
        self.color_for_elapsed_percent(color, elapsed_percent)
    }
}

impl From<(ColorModulation, Beats)> for ColorModulator {
    fn from((modulation, meter_length): (ColorModulation, Beats)) -> ColorModulator {
        ColorModulator::new_static(modulation, meter_length)
    }
}

// Utilities
pub fn intensity(dimmer: f64, intensity: f64) -> f64 {
    1.0 - intensity + dimmer * intensity
}

pub fn color_intensity(color: Hsl64, effected_color: Hsl64, effect_intensity: f64) -> Hsl64 {
    color.mix(&effected_color, effect_intensity)
}
