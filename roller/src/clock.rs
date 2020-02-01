use derive_more::{Add, Constructor, From, Into};
use std::time::{Duration, Instant};

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Add, From, Into, Constructor)]
pub struct Beats(f64);

pub struct Clock {
    started_at: Instant,
    bpm: f64,
    taps: Vec<Instant>,
}
impl Clock {
    pub fn new(bpm: f64) -> Clock {
        Clock {
            bpm,
            started_at: Instant::now(),
            taps: Vec::new(),
        }
    }
    pub fn tap(&mut self, now: Instant) {
        // If last tap was more than 3 seconds ago, clear the taps
        if let Some(last_tap) = self.taps.last() {
            if (now - *last_tap) > Duration::from_secs(2) {
                dbg!(&self.taps);
                self.taps.clear();
                dbg!(&self.taps);
            }
        }

        self.taps.push(now);

        if self.taps.len() > 4 {
            let time_elapsed = now - *self.taps.first().unwrap();

            self.started_at = now;

            let beat_duration_secs =
                (time_elapsed.as_millis() as f64 / 1000.0) / self.taps.len() as f64;
            self.bpm = 60.0 / beat_duration_secs;
        }
    }
    pub fn bpm(&self) -> f64 {
        self.bpm
    }
    fn secs_elapsed(&self) -> f64 {
        let elapsed_duration = Instant::now() - self.started_at;
        elapsed_duration.as_millis() as f64 / 1000.0
    }
    pub fn snapshot(&self) -> ClockSnapshot {
        ClockSnapshot {
            secs_elapsed: self.secs_elapsed(),
            bpm: self.bpm,
        }
    }
}

pub struct ClockSnapshot {
    secs_elapsed: f64,
    bpm: f64,
}
impl ClockSnapshot {
    pub fn bpm(&self) -> f64 {
        self.bpm
    }
    pub fn secs_elapsed(&self) -> f64 {
        self.secs_elapsed
    }
    pub fn secs_per_meter(&self, beats: Beats) -> f64 {
        60.0 / self.bpm * f64::from(beats)
    }
    pub fn meter_progress(&self, beats: Beats) -> f64 {
        let secs_elapsed = self.secs_elapsed();
        let secs_per_meter = self.secs_per_meter(beats);

        1.0 / secs_per_meter * (secs_elapsed % secs_per_meter)
    }
}
