use std::time::Instant;

pub struct Clock {
    started_at: Instant,
    bpm: f64,
}
impl Clock {
    pub fn new(bpm: f64) -> Clock {
        Clock {
            bpm,
            started_at: Instant::now(),
        }
    }
    pub fn secs_elapsed(&self) -> f64 {
        let elapsed_duration = Instant::now() - self.started_at;
        elapsed_duration.as_millis() as f64 / 1000.0
    }
    pub fn secs_per_meter(&self, beats: f64) -> f64 {
        60.0 / self.bpm * beats
    }
    pub fn meter_progress(&self, beats: f64) -> f64 {
        let secs_elapsed = self.secs_elapsed();
        let secs_per_meter = self.secs_per_meter(beats);

        1.0 / secs_per_meter * (secs_elapsed % secs_per_meter)
    }
}
