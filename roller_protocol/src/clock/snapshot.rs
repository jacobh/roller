use crate::clock::{Beats, Rate};
use std::borrow::Cow;

#[derive(Debug, Clone, PartialEq)]
pub struct ClockSnapshot {
    pub secs_elapsed: f64,
    pub bpm: f64,
}
impl ClockSnapshot {
    pub fn with_rate(&self, rate: Rate) -> Cow<ClockSnapshot> {
        if rate.is_one() {
            Cow::Borrowed(self)
        } else {
            Cow::Owned(ClockSnapshot {
                secs_elapsed: self.secs_elapsed * f64::from(rate),
                bpm: self.bpm,
            })
        }
    }
    pub fn shift(&self, beats: Beats) -> Cow<ClockSnapshot> {
        if beats.is_zero() {
            Cow::Borrowed(self)
        } else {
            let secs_per_beat = 60.0 / self.bpm;
            let secs_to_shift = secs_per_beat * f64::from(beats);

            Cow::Owned(ClockSnapshot {
                secs_elapsed: self.secs_elapsed + secs_to_shift,
                bpm: self.bpm,
            })
        }
    }
    pub fn secs_elapsed(&self) -> f64 {
        self.secs_elapsed
    }
    pub fn secs_per_meter(&self, meter_length: Beats) -> f64 {
        60.0 / self.bpm * f64::from(meter_length)
    }
    pub fn meter_elapsed_percent(&self, meter_length: Beats) -> f64 {
        let secs_elapsed = self.secs_elapsed();
        let secs_per_meter = self.secs_per_meter(meter_length);

        1.0 / secs_per_meter * (secs_elapsed % secs_per_meter)
    }
}
