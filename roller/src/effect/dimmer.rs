use ordered_float::OrderedFloat;

use crate::{
    clock::{Beats, ClockOffset, ClockSnapshot},
    fixture::Fixture,
    effect::Waveform,
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
}
