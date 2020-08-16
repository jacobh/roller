use async_std::prelude::*;

use std::time::{Duration, Instant};

use roller_protocol::clock::{Beats, ClockSnapshot};

fn duration_as_secs(duration: Duration) -> f64 {
    duration.as_micros() as f64 / 1_000_000.0
}

#[derive(Debug)]
pub enum ClockEvent {
    BpmChanged(f64),
    Tap(Instant),
}

#[derive(Debug, Clone, PartialEq)]
enum ClockState {
    Manual { taps: Vec<Instant> },
    Automatic,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Clock {
    started_at: Instant,
    bpm: f64,
    state: ClockState,
}
impl Clock {
    pub fn new(bpm: f64) -> Clock {
        Clock {
            bpm,
            started_at: Instant::now(),
            state: ClockState::Manual { taps: Vec::new() },
        }
    }
    pub fn apply_event(&mut self, event: ClockEvent) {
        match event {
            ClockEvent::Tap(now) => {
                match self.state {
                    ClockState::Manual { ref mut taps } => {
                        // If last tap was more than 1 second ago, clear the taps
                        if let Some(last_tap) = taps.last() {
                            if (now - *last_tap) > Duration::from_secs(1) {
                                dbg!(&taps);
                                taps.clear();
                                dbg!(&taps);
                            }
                        }

                        taps.push(now);

                        if taps.len() >= 4 {
                            let time_elapsed = now - *taps.first().unwrap();
                            let beat_duration_secs =
                                duration_as_secs(time_elapsed) / (taps.len() - 1) as f64;

                            self.started_at = now;
                            self.bpm = 60.0 / beat_duration_secs;
                        }
                    }
                    ClockState::Automatic => {
                        self.started_at = now;
                    }
                }
            }
            ClockEvent::BpmChanged(bpm) => {
                // Periodically reset the start time to avoid glitchiness when tempo slightly drifts
                let snapshot = self.snapshot();
                let beat_secs = snapshot.secs_per_meter(Beats::new(1.0));

                if snapshot.secs_elapsed() - (beat_secs * 48.0) >= 0.0 {
                    self.started_at += Duration::from_secs_f64(beat_secs * 48.0);
                }

                self.state = ClockState::Automatic;
                self.bpm = bpm;
            }
        }
    }
    pub fn started_at(&self) -> Instant {
        self.started_at
    }
    pub fn bpm(&self) -> f64 {
        self.bpm
    }
    pub fn secs_elapsed(&self) -> f64 {
        duration_as_secs(Instant::now() - self.started_at())
    }
    pub fn snapshot(&self) -> ClockSnapshot {
        ClockSnapshot {
            secs_elapsed: self.secs_elapsed(),
            bpm: self.bpm(),
        }
    }
}

static PULSES_PER_QUARTER_NOTE: usize = 24;

pub fn midi_clock_events(name: &str) -> Result<impl Stream<Item = ClockEvent>, midi::MidiIoError> {
    let input = midi::MidiInput::new(name)?;
    let mut pulses: Vec<Instant> = Vec::with_capacity(PULSES_PER_QUARTER_NOTE);

    Ok(input
        .filter(|midi_event| midi_event == &midi::MidiEvent::TimingClock)
        .filter_map(move |_| {
            pulses.push(Instant::now());

            if pulses.len() == PULSES_PER_QUARTER_NOTE {
                let first_pulse = pulses[0];
                let last_pulse = pulses[PULSES_PER_QUARTER_NOTE - 1];

                let duration = last_pulse - first_pulse;
                let secs_per_beat = duration_as_secs(duration) / (pulses.len() - 1) as f64 * 24.0;
                let bpm = 60.0 / secs_per_beat;

                pulses.clear();
                dbg!(Some(ClockEvent::BpmChanged(bpm)))
            } else {
                None
            }
        }))
}
