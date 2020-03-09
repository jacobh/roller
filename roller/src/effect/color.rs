use ordered_float::OrderedFloat;
use palette::{Hue, Mix};

use crate::{
    clock::{Beats, ClockOffset, ClockSnapshot},
    color::{Color, Hsl64},
    effect::{ModulatorSteps, Modulator, Waveform},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ColorEffect {
    steps: ModulatorSteps<ColorModulator>,
    pub clock_offset: Option<ClockOffset>,
}
impl ColorEffect {
    pub fn new(steps: Vec<ColorModulator>, clock_offset: Option<ClockOffset>) -> ColorEffect {
        ColorEffect {
            steps: ModulatorSteps::new(steps),
            clock_offset,
        }
    }
    pub fn color(
        &self,
        color: Hsl64,
        secondary_color: Option<Hsl64>,
        clock: &ClockSnapshot,
    ) -> Hsl64 {
        let (step, elapsed_percent) = self.steps.current_step(clock);
        step.color_for_elapsed_percent(color, secondary_color, elapsed_percent)
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
    ToSecondaryColor,
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
    fn color_for_elapsed_percent(
        &self,
        color: Hsl64,
        secondary_color: Option<Hsl64>,
        elapsed_percent: f64,
    ) -> Hsl64 {
        match self.modulation {
            ColorModulation::HueShift(shift_degrees) => {
                color.shift_hue(self.waveform.apply(elapsed_percent) * shift_degrees.into_inner())
            }
            ColorModulation::ToSecondaryColor => {
                let degrees_to_secondary = secondary_color
                    .map(|secondary_color| (secondary_color.hue - color.hue).to_degrees())
                    .unwrap_or(0.0);

                color.shift_hue(degrees_to_secondary * self.waveform.apply(elapsed_percent))
            }
            ColorModulation::White => {
                color.mix(&Color::White.to_hsl(), self.waveform.apply(elapsed_percent))
            }
            ColorModulation::NoOp => color,
        }
    }
}
impl Modulator for ColorModulator {
    fn meter_length(&self) -> Beats {
        self.meter_length
    }
}

impl From<(ColorModulation, Beats)> for ColorModulator {
    fn from((modulation, meter_length): (ColorModulation, Beats)) -> ColorModulator {
        ColorModulator::new_static(modulation, meter_length)
    }
}
