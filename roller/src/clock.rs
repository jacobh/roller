use async_std::prelude::*;
use std::time::{Duration, Instant};

use roller_protocol::clock::ClockEvent;

fn duration_as_secs(duration: Duration) -> f64 {
    duration.as_micros() as f64 / 1_000_000.0
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
