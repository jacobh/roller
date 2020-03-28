use async_std::prelude::*;
use derive_more::{From, Into};
use ordered_float::OrderedFloat;
use rand::{seq::SliceRandom, thread_rng};
use std::borrow::Cow;
use std::collections::VecDeque;
use std::iter::Sum;
use std::ops::{Add, Mul, Sub};
use std::time::{Duration, Instant};

use crate::fixture::Fixture;

fn duration_as_secs(duration: Duration) -> f64 {
    duration.as_micros() as f64 / 1_000_000.0
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, From, Into)]
pub struct Beats(OrderedFloat<f64>);
impl Beats {
    pub fn new(x: impl Into<OrderedFloat<f64>>) -> Beats {
        Beats(x.into())
    }
    pub fn is_zero(&self) -> bool {
        self.0.into_inner() == 0.0
    }
}

impl Add for Beats {
    type Output = Beats;
    fn add(self, other: Beats) -> Beats {
        Beats::new(self.0.into_inner() + other.0.into_inner())
    }
}

impl Sub for Beats {
    type Output = Beats;
    fn sub(self, other: Beats) -> Beats {
        Beats::new(self.0.into_inner() - other.0.into_inner())
    }
}

impl Mul<f64> for Beats {
    type Output = Beats;
    fn mul(self, other: f64) -> Beats {
        Beats::new(self.0.into_inner() * other)
    }
}

impl Sum<Beats> for Beats {
    fn sum<I>(iter: I) -> Beats
    where
        I: Iterator<Item = Beats>,
    {
        iter.fold(Beats::new(0.0), Add::add)
    }
}

impl From<Beats> for f64 {
    fn from(beats: Beats) -> f64 {
        beats.0.into()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, From, Into)]
pub struct Rate(OrderedFloat<f64>);
impl Rate {
    pub fn new(x: impl Into<OrderedFloat<f64>>) -> Rate {
        Rate(x.into())
    }
    pub fn is_one(&self) -> bool {
        self.0.into_inner() == 1.0
    }
}
impl Default for Rate {
    fn default() -> Rate {
        Rate::new(1.0)
    }
}
impl From<Rate> for f64 {
    fn from(rate: Rate) -> f64 {
        rate.0.into_inner()
    }
}

#[derive(Debug, Clone, PartialEq)]
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
        // If last tap was more than 1 second ago, clear the taps
        if let Some(last_tap) = self.taps.last() {
            if (now - *last_tap) > Duration::from_secs(1) {
                dbg!(&self.taps);
                self.taps.clear();
                dbg!(&self.taps);
            }
        }

        self.taps.push(now);

        if self.taps.len() >= 4 {
            let time_elapsed = now - *self.taps.first().unwrap();

            self.started_at = now;

            let beat_duration_secs = duration_as_secs(time_elapsed) / (self.taps.len() - 1) as f64;
            self.bpm = 60.0 / beat_duration_secs;
        }
    }
    pub fn bpm(&self) -> f64 {
        self.bpm
    }
    fn secs_elapsed(&self) -> f64 {
        let elapsed_duration = Instant::now() - self.started_at;
        elapsed_duration.as_micros() as f64 / 1_000_000.0
    }
    pub fn snapshot(&self) -> ClockSnapshot {
        ClockSnapshot {
            secs_elapsed: self.secs_elapsed(),
            bpm: self.bpm,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClockSnapshot {
    secs_elapsed: f64,
    bpm: f64,
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub enum ClockOffsetMode {
    GroupId,
    FixtureIndex,
    Random,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClockOffset {
    mode: ClockOffsetMode,
    offset: Beats,
    seed: [u8; 32],
}
impl ClockOffset {
    pub fn new(mode: ClockOffsetMode, offset: Beats) -> ClockOffset {
        // create a seed array of the numbers 0 - 31
        let mut rng = thread_rng();
        let mut seed = [0u8; 32];
        for (i, x) in seed.iter_mut().enumerate() {
            *x = i as u8;
        }
        seed.shuffle(&mut rng);

        ClockOffset { mode, offset, seed }
    }
    pub fn offset_for_fixture(&self, fixture: &Fixture, fixtures: &[Fixture]) -> Beats {
        match self.mode {
            ClockOffsetMode::GroupId => {
                self.offset
                    * fixture
                        .group_id
                        .map(|group_id| usize::from(group_id) as f64 - 1.0)
                        .unwrap_or(0.0)
            }
            ClockOffsetMode::FixtureIndex => {
                let fixture_idx = fixtures.iter().position(|x| x == fixture).unwrap();
                self.offset * fixture_idx as f64
            }
            ClockOffsetMode::Random => {
                let fixture_idx = fixtures.iter().position(|x| x == fixture).unwrap();
                self.offset * self.seed[fixture_idx % 32] as f64
            }
        }
    }
    pub fn offsetted_for_fixture<'a>(
        &self,
        clock: &'a ClockSnapshot,
        fixture: &Fixture,
        fixtures: &[Fixture],
    ) -> Cow<'a, ClockSnapshot> {
        clock.shift(self.offset_for_fixture(fixture, fixtures))
    }
}

pub fn offsetted_for_fixture<'a>(
    clock_offset: Option<&ClockOffset>,
    clock: &'a ClockSnapshot,
    fixture: &Fixture,
    fixtures: &[Fixture],
) -> Cow<'a, ClockSnapshot> {
    match clock_offset {
        Some(clock_offset) => clock_offset.offsetted_for_fixture(clock, fixture, fixtures),
        None => Cow::Borrowed(clock),
    }
}

static PULSES_PER_QUARTER_NOTE: usize = 24;

pub struct MidiClock {
    // input: midi::MidiInput,
}
impl MidiClock {
    pub fn new(name: &str) -> Result<MidiClock, midi::MidiIoError> {
        let input = midi::MidiInput::new(name)?;

        async_std::task::spawn(async move {
            let mut events = input.events();
            let mut pulses: VecDeque<Instant> = VecDeque::new();
            while let Some(event) = events.next().await {
                if event == midi::MidiEvent::TimingClock {
                    pulses.push_back(Instant::now());

                    while pulses.len() > PULSES_PER_QUARTER_NOTE {
                        pulses.pop_front();
                    }

                    if let (Some(first_quarter_note), Some(last_quarter_note)) =
                        (pulses.front(), pulses.back())
                    {
                        let duration = *last_quarter_note - *first_quarter_note;
                        let secs_per_beat =
                            duration_as_secs(duration) / (pulses.len() - 1) as f64 * 24.0;

                        dbg!(60.0 / secs_per_beat);
                    }
                }
            }
        });

        Ok(MidiClock {})
    }
}
