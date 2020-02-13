use ordered_float::OrderedFloat;
use palette::{Hue, Mix, RgbHue};
use std::f64::consts::PI;

use crate::{
    clock::{Beats, ClockOffset, ClockSnapshot},
    color::{Color, Hsl64},
    fixture::Fixture,
};

// TODO name subject to change
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DimmerModifier {
    Effect(DimmerEffect),
    Sequence(DimmerSequence),
}
impl DimmerModifier {
    fn dimmer(&self, clock: &ClockSnapshot) -> f64 {
        match self {
            DimmerModifier::Effect(effect) => effect.dimmer(clock),
            DimmerModifier::Sequence(sequence) => sequence.dimmer(clock),
        }
    }
    pub fn offset_dimmer(
        &self,
        clock: &ClockSnapshot,
        fixture: &Fixture,
        fixtures: &[Fixture],
    ) -> f64 {
        // TODO clock offsets for dimmer effects
        let clock_offset = match self {
            DimmerModifier::Effect(_) => None,
            DimmerModifier::Sequence(sequence) => sequence.clock_offset.as_ref(),
        };

        match clock_offset {
            Some(clock_offset) => {
                let offset = clock_offset.offset_for_fixture(fixture, fixtures);
                self.dimmer(&clock.shift(offset))
            }
            None => self.dimmer(clock),
        }
    }
}
impl From<DimmerEffect> for DimmerModifier {
    fn from(effect: DimmerEffect) -> DimmerModifier {
        DimmerModifier::Effect(effect)
    }
}
impl From<DimmerSequence> for DimmerModifier {
    fn from(sequence: DimmerSequence) -> DimmerModifier {
        DimmerModifier::Sequence(sequence)
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
pub struct DimmerEffect {
    effect: Effect,
    meter_length: Beats,
    scale: DimmerScale,
}
impl DimmerEffect {
    pub fn new(effect: Effect, meter_length: Beats, scale: impl Into<DimmerScale>) -> DimmerEffect {
        DimmerEffect {
            meter_length,
            scale: scale.into(),
            effect: effect,
        }
    }
    fn dimmer_for_elapsed_percent(&self, elapsed_percent: f64) -> f64 {
        self.scale.scale(self.effect.apply(elapsed_percent))
    }
    pub fn dimmer(&self, clock: &ClockSnapshot) -> f64 {
        let elapsed_percent = clock.meter_elapsed_percent(self.meter_length);
        self.dimmer_for_elapsed_percent(elapsed_percent)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DimmerSequence {
    steps: Vec<DimmerEffect>,
    clock_offset: Option<ClockOffset>,
}
impl DimmerSequence {
    pub fn new(steps: Vec<DimmerEffect>, clock_offset: Option<ClockOffset>) -> DimmerSequence {
        DimmerSequence {
            steps,
            clock_offset,
        }
    }
    fn total_length(&self) -> Beats {
        self.steps
            .iter()
            .map(|dimmer_effect| dimmer_effect.meter_length)
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Effect {
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
impl Effect {
    fn apply(self, x: f64) -> f64 {
        match self {
            Effect::SawUp => saw_up(x),
            Effect::SawDown => saw_down(x),
            Effect::TriangleDown => triangle_down(x),
            Effect::SineUp => sine_up(x),
            Effect::SineDown => sine_down(x),
            Effect::HalfSineUp => half_sine_up(x),
            Effect::HalfSineDown => half_sine_down(x),
            Effect::ShortSquarePulse => short_square_pulse(x),
            Effect::On => 1.0,
            Effect::Off => 0.0,
        }
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
pub enum ColorEffectMode {
    HueShift(OrderedFloat<f64>),
    White,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ColorEffect {
    mode: ColorEffectMode,
    effect: Effect,
    meter_length: Beats,
    clock_offset: Option<ClockOffset>,
}
impl ColorEffect {
    pub fn new(
        mode: ColorEffectMode,
        effect: Effect,
        meter_length: Beats,
        clock_offset: Option<ClockOffset>,
    ) -> ColorEffect {
        ColorEffect {
            mode,
            effect,
            meter_length,
            clock_offset,
        }
    }
    pub fn color(&self, color: Hsl64, clock: &ClockSnapshot) -> Hsl64 {
        let elapsed_percent = clock.meter_elapsed_percent(self.meter_length);

        match self.mode {
            ColorEffectMode::HueShift(shift_degrees) => {
                color.shift_hue(RgbHue::<f64>::from_degrees(
                    self.effect.apply(elapsed_percent) * shift_degrees.into_inner(),
                ))
            }
            ColorEffectMode::White => {
                color.mix(&Color::White.to_hsl(), self.effect.apply(elapsed_percent))
            }
        }
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

// Utilities
pub fn intensity(dimmer: f64, intensity: f64) -> f64 {
    1.0 - intensity + dimmer * intensity
}

pub fn color_intensity(color: Hsl64, effected_color: Hsl64, effect_intensity: f64) -> Hsl64 {
    color.mix(&effected_color, effect_intensity)
}
