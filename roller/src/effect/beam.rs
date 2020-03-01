use crate::{
    clock::{Beats, ClockOffset, ClockSnapshot},
    effect::Waveform,
    fixture::Fixture,
};

fn percent_contained(a: (f64, f64), b: (f64, f64)) -> f64 {
    let b_range = b.1 - b.0;
    let lower_bounds = f64::max(a.0, b.0);
    let upper_bounds = f64::min(a.1, b.1);

    if b_range > 0.0 {
        let contained_range = f64::max(upper_bounds - lower_bounds, 0.0);
        1.0 / b_range * contained_range
    } else {
        if lower_bounds == upper_bounds {
            1.0
        } else {
            0.0
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BeamRangeStop {
    low: f64,
    high: f64,
}
impl BeamRangeStop {
    fn new(a: f64, b: f64) -> BeamRangeStop {
        let (low, high) = if a > b { (b, a) } else { (a, b) };
        BeamRangeStop {low, high}
    }
}

impl From<&(f64, f64)> for BeamRangeStop {
    fn from((a, b): &(f64, f64)) -> BeamRangeStop {
        BeamRangeStop::new(*a, *b)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BeamRange {
    stops: Vec<BeamRangeStop>,
}
impl BeamRange {
    fn new(stops: impl IntoIterator<Item = impl Into<BeamRangeStop>>) -> BeamRange {
        BeamRange {
            stops: stops.into_iter().map(|stop| stop.into()).collect(),
        }
    }
    pub fn beam_dimmers(&self, beam_count: usize) -> Vec<f64> {
        let beam_width = 1.0 / beam_count as f64;

        (0..beam_count)
            .into_iter()
            .map(|beam_idx| {
                let beam_min = beam_idx as f64 * beam_width;
                let beam_max = (beam_idx + 1) as f64 * beam_width;

                f64::min(
                    self.stops
                        .iter()
                        .map(|stop| percent_contained((beam_min, beam_max), (stop.low, stop.high)))
                        .sum(),
                    1.0,
                )
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BeamEffect {
    steps: Vec<BeamModulator>,
    clock_offset: Option<ClockOffset>,
}
impl BeamEffect {
    pub fn new(steps: Vec<BeamModulator>, clock_offset: Option<ClockOffset>) -> BeamEffect {
        BeamEffect {
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
    pub fn beam(&self, clock: &ClockSnapshot) -> BeamRange {
        let length = self.total_length();
        let elapsed_percent = clock.meter_elapsed_percent(length);
        let mut elapsed_beats = length * elapsed_percent;

        for step in self.steps.iter() {
            if step.meter_length >= elapsed_beats {
                return step.beam_for_elapsed_percent(
                    1.0 / f64::from(step.meter_length) * f64::from(elapsed_beats),
                );
            } else {
                elapsed_beats = elapsed_beats - step.meter_length;
            }
        }

        unreachable!()
    }
    pub fn offset_beam(
        &self,
        clock: &ClockSnapshot,
        fixture: &Fixture,
        fixtures: &[Fixture],
    ) -> BeamRange {
        match &self.clock_offset {
            Some(clock_offset) => {
                let offset = clock_offset.offset_for_fixture(fixture, fixtures);
                self.beam(&clock.shift(offset))
            }
            None => self.beam(clock),
        }
    }
}

impl From<BeamModulator> for BeamEffect {
    fn from(modulator: BeamModulator) -> BeamEffect {
        BeamEffect::new(vec![modulator], None)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModulatorDirection {
    BottomToTop,
    ToCenter,
    FromCenter,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BeamModulator {
    waveform: Waveform,
    meter_length: Beats,
    direction: ModulatorDirection,
}
impl BeamModulator {
    pub fn new(waveform: Waveform, meter_length: Beats) -> BeamModulator {
        BeamModulator {
            meter_length,
            waveform,
            direction: ModulatorDirection::ToCenter,
        }
    }
    fn beam_for_elapsed_percent(&self, elapsed_percent: f64) -> BeamRange {
        let x = self.waveform.apply(elapsed_percent);
        let low = f64::max(x - 0.1, 0.0);
        let high = f64::min(x + 0.1, 1.0);

        match self.direction {
            ModulatorDirection::BottomToTop => BeamRange::new(&[(low, high)]),
            ModulatorDirection::FromCenter => {
                let low = low / 2.0;
                let high = high / 2.0;

                BeamRange::new(&[(0.5 - low, 0.5 - high), (0.5 + low, 0.5 + high)])
            }
            ModulatorDirection::ToCenter => {
                let low = low / 2.0;
                let high = high / 2.0;

                BeamRange::new(&[(low, high), (1.0 - low, 1.0 - high)])
            }
        }
    }
}
