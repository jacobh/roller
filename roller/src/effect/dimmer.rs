use ordered_float::OrderedFloat;

use crate::{
    clock::{Beats, ClockOffset, ClockSnapshot},
    effect::{Step, Steps, Waveform},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DimmerEffect {
    steps: Steps<DimmerModulator>,
    pub clock_offset: Option<ClockOffset>,
}
impl DimmerEffect {
    pub fn new(steps: Vec<DimmerModulator>, clock_offset: Option<ClockOffset>) -> DimmerEffect {
        DimmerEffect {
            steps: Steps::new(steps),
            clock_offset,
        }
    }
    pub fn dimmer(&self, clock: &ClockSnapshot) -> f64 {
        let (step, elapsed_percent) = self.steps.current_step(clock);
        step.dimmer_for_elapsed_percent(elapsed_percent)
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
}
impl Step for DimmerModulator {
    fn meter_length(&self) -> Beats {
        self.meter_length
    }
}
