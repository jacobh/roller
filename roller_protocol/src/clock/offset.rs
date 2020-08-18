use std::borrow::Cow;

use itertools::Itertools;
use rand::{seq::SliceRandom, thread_rng};
use serde::{Deserialize, Serialize};

use crate::{
    clock::{Beats, ClockSnapshot},
    effect::EffectDirection,
    fixture::FixtureParams,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[allow(dead_code)]
pub enum ClockOffsetMode {
    GroupId,
    FixtureIndex,
    Random,
    Location(EffectDirection),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    pub fn offset_for_fixture(&self, fixture: &FixtureParams, fixtures: &[FixtureParams]) -> Beats {
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
            ClockOffsetMode::Location(direction) => {
                let fixture_locations = fixtures
                    .iter()
                    .flat_map(|fixture| fixture.location.as_ref());

                match (fixture.location.as_ref(), direction) {
                    (Some(location), EffectDirection::LeftToRight) => {
                        let location_idx = fixture_locations
                            .map(|location| location.x)
                            .unique()
                            .sorted()
                            .rev()
                            .position(|x| x == location.x)
                            .unwrap();

                        self.offset * location_idx as f64
                    }
                    (Some(location), EffectDirection::BottomToTop) => {
                        let location_idx = fixture_locations
                            .map(|location| location.y)
                            .unique()
                            .sorted()
                            .rev()
                            .position(|y| y == location.y)
                            .unwrap();

                        self.offset * location_idx as f64
                    }
                    (Some(location), EffectDirection::ToCenter)
                    | (Some(location), EffectDirection::FromCenter) => {
                        let xs = fixture_locations
                            .map(|location| location.x)
                            .unique()
                            .sorted()
                            .collect_vec();
                        let len = xs.len();

                        let mut pairs = xs
                            .clone()
                            .into_iter()
                            .zip(xs.into_iter().rev())
                            .take((len + 1) / 2);

                        let location_idx = match direction {
                            EffectDirection::ToCenter => pairs
                                .rev()
                                .position(|(x1, x2)| x1 == location.x || x2 == location.x)
                                .unwrap(),
                            EffectDirection::FromCenter => pairs
                                .position(|(x1, x2)| x1 == location.x || x2 == location.x)
                                .unwrap(),
                            _ => unreachable!(),
                        };

                        self.offset * location_idx as f64
                    }
                    (None, _) => Beats::new(0.0),
                }
            }
        }
    }
    pub fn offsetted_for_fixture<'a>(
        &self,
        clock: &'a ClockSnapshot,
        fixture: &FixtureParams,
        fixtures: &[FixtureParams],
    ) -> Cow<'a, ClockSnapshot> {
        clock.shift(self.offset_for_fixture(fixture, fixtures))
    }
}

pub fn offsetted_for_fixture<'a>(
    clock_offset: Option<&ClockOffset>,
    clock: &'a ClockSnapshot,
    fixture: &FixtureParams,
    fixtures: &[FixtureParams],
) -> Cow<'a, ClockSnapshot> {
    match clock_offset {
        Some(clock_offset) => clock_offset.offsetted_for_fixture(clock, fixture, fixtures),
        None => Cow::Borrowed(clock),
    }
}
