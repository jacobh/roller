use ordered_float::OrderedFloat;

use roller_protocol::{clock::Beats, position::Position};

use crate::{
    clock::{ClockOffset, ClockSnapshot},
    effect::Waveform,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PositionEffect {
    pan: Option<PositionModulator>,
    tilt: Option<PositionModulator>,
    pub clock_offset: Option<ClockOffset>,
}
impl PositionEffect {
    pub fn new(
        pan: Option<PositionModulator>,
        tilt: Option<PositionModulator>,
        clock_offset: Option<ClockOffset>,
    ) -> PositionEffect {
        PositionEffect {
            pan,
            tilt,
            clock_offset,
        }
    }
    pub fn position(&self, clock: &ClockSnapshot) -> Position {
        let pan = self.pan.as_ref().map(|pan| pan.axis(clock)).unwrap_or(0.0);
        let tilt = self
            .tilt
            .as_ref()
            .map(|tilt| tilt.axis(clock))
            .unwrap_or(0.0);

        Position::new(pan, tilt)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PositionModulator {
    waveform: Waveform,
    meter_length: Beats,
    range: OrderedFloat<f64>,
}
impl PositionModulator {
    pub fn new(
        waveform: Waveform,
        meter_length: Beats,
        range: impl Into<OrderedFloat<f64>>,
    ) -> PositionModulator {
        PositionModulator {
            meter_length,
            waveform,
            range: range.into(),
        }
    }
    fn axis(&self, clock: &ClockSnapshot) -> f64 {
        let elapsed_percent = clock.meter_elapsed_percent(self.meter_length);
        self.axis_for_elapsed_percent(elapsed_percent)
    }
    fn axis_for_elapsed_percent(&self, elapsed_percent: f64) -> f64 {
        // waveform value in range -1.0 - 1.0
        let value = (self.waveform.apply(elapsed_percent) * 2.0) - 1.0;

        self.range.into_inner() / 2.0 * value
    }
}
