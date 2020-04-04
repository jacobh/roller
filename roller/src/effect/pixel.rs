use crate::{
    clock::{Beats, ClockOffset, ClockSnapshot},
    effect::{EffectDirection, Modulator, ModulatorSteps, Waveform},
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
pub struct PixelRange {
    low: f64,
    high: f64,
}
impl PixelRange {
    fn new(a: f64, b: f64) -> PixelRange {
        let (low, high) = if a > b { (b, a) } else { (a, b) };
        PixelRange { low, high }
    }
}

impl From<&(f64, f64)> for PixelRange {
    fn from((a, b): &(f64, f64)) -> PixelRange {
        PixelRange::new(*a, *b)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PixelRangeSet {
    ranges: Vec<PixelRange>,
}
impl PixelRangeSet {
    fn new(ranges: impl IntoIterator<Item = impl Into<PixelRange>>) -> PixelRangeSet {
        PixelRangeSet {
            ranges: ranges.into_iter().map(|stop| stop.into()).collect(),
        }
    }
    pub fn pixel_dimmers(&self, pixel_count: usize) -> Vec<f64> {
        let pixel_width = 1.0 / pixel_count as f64;

        (0..pixel_count)
            .into_iter()
            .map(|pixel_idx| {
                let pixel_min = pixel_idx as f64 * pixel_width;
                let pixel_max = (pixel_idx + 1) as f64 * pixel_width;

                f64::min(
                    self.ranges
                        .iter()
                        .map(|stop| {
                            percent_contained((pixel_min, pixel_max), (stop.low, stop.high))
                        })
                        .sum(),
                    1.0,
                )
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PixelEffect {
    steps: ModulatorSteps<PixelModulator>,
    pub clock_offset: Option<ClockOffset>,
}
impl PixelEffect {
    pub fn new(steps: Vec<PixelModulator>, clock_offset: Option<ClockOffset>) -> PixelEffect {
        PixelEffect {
            steps: ModulatorSteps::new(steps),
            clock_offset,
        }
    }
    pub fn pixel_range_set(&self, clock: &ClockSnapshot) -> PixelRangeSet {
        let (step, elapsed_percent) = self.steps.current_step(clock);
        step.pixel_range_set_for_elapsed_percent(elapsed_percent)
    }
}

impl From<PixelModulator> for PixelEffect {
    fn from(modulator: PixelModulator) -> PixelEffect {
        PixelEffect::new(vec![modulator], None)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PixelModulator {
    waveform: Waveform,
    meter_length: Beats,
    direction: EffectDirection,
}
impl PixelModulator {
    pub fn new(
        waveform: Waveform,
        meter_length: Beats,
        direction: EffectDirection,
    ) -> PixelModulator {
        PixelModulator {
            meter_length,
            waveform,
            direction,
        }
    }
    fn pixel_range_set_for_elapsed_percent(&self, elapsed_percent: f64) -> PixelRangeSet {
        let x = self.waveform.apply(elapsed_percent);
        let low = f64::max(x - 0.1, 0.0);
        let high = f64::min(x + 0.1, 1.0);

        match self.direction {
            EffectDirection::BottomToTop => PixelRangeSet::new(&[(low, high)]),
            EffectDirection::FromCenter => {
                let low = low / 2.0;
                let high = high / 2.0;

                PixelRangeSet::new(&[(0.5 - low, 0.5 - high), (0.5 + low, 0.5 + high)])
            }
            EffectDirection::ToCenter => {
                let low = low / 2.0;
                let high = high / 2.0;

                PixelRangeSet::new(&[(low, high), (1.0 - low, 1.0 - high)])
            }
            EffectDirection::LeftToRight => PixelRangeSet::new(&[(0.0, 1.0)]),
        }
    }
}
impl Modulator for PixelModulator {
    fn meter_length(&self) -> Beats {
        self.meter_length
    }
}
