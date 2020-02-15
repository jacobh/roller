use ordered_float::OrderedFloat;
use palette::{Hue, Mix, RgbHue};

use crate::{
    clock::{Beats, ClockOffset, ClockSnapshot},
    color::{Color, Hsl64},
    effect::Waveform,
    fixture::Fixture,
};

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
}

impl From<(ColorModulation, Beats)> for ColorModulator {
    fn from((modulation, meter_length): (ColorModulation, Beats)) -> ColorModulator {
        ColorModulator::new_static(modulation, meter_length)
    }
}
