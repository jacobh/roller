use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std::ops::Add;

use crate::{fixture::FixtureParams, utils::clamp};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Position {
    pan: OrderedFloat<f64>,
    tilt: OrderedFloat<f64>,
}
impl Position {
    pub fn new(pan: f64, tilt: f64) -> Position {
        Position {
            pan: pan.into(),
            tilt: tilt.into(),
        }
    }
    pub fn pan(&self) -> f64 {
        self.pan.into_inner()
    }
    pub fn tilt(&self) -> f64 {
        self.tilt.into_inner()
    }
    pub fn inverted_pan(mut self) -> Position {
        self.pan = OrderedFloat::from(-*self.pan);
        self
    }
}
impl Default for Position {
    fn default() -> Position {
        Position::new(0.0, 0.0)
    }
}

impl Add for Position {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Position::new(self.pan() + other.pan(), self.tilt() + other.tilt())
    }
}

impl From<(f64, f64)> for Position {
    fn from((pan, tilt): (f64, f64)) -> Position {
        Position::new(pan, tilt)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum BasePositionMode {
    Default,
    MirrorPan,
}
impl Default for BasePositionMode {
    fn default() -> BasePositionMode {
        BasePositionMode::Default
    }
}

#[derive(
    Debug, Default, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
pub struct BasePosition {
    pub position: Position,
    pub mode: BasePositionMode,
}
impl BasePosition {
    pub fn new(position: Position, mode: BasePositionMode) -> BasePosition {
        BasePosition { position, mode }
    }
    pub fn for_fixture(&self, fixture: &FixtureParams, fixtures: &[&FixtureParams]) -> Position {
        // Hackily find the index of this moving fixture and use that for mirroring.
        // Ultimately we need a `location` attribute on a fixture
        let moving_fixtures = fixtures
            .iter()
            .filter(|fixture| fixture.profile.is_positionable());
        let fixture_i = moving_fixtures
            .enumerate()
            .find(|(_, f)| **f == fixture)
            .map(|(i, _)| i)
            .unwrap_or(0);

        match self.mode {
            BasePositionMode::Default => self.position,
            BasePositionMode::MirrorPan => {
                if fixture_i % 2 == 0 {
                    self.position
                } else {
                    self.position.inverted_pan()
                }
            }
        }
    }
}

impl<T> From<(T, BasePositionMode)> for BasePosition
where
    T: Into<Position>,
{
    fn from((position, mode): (T, BasePositionMode)) -> BasePosition {
        BasePosition::new(position.into(), mode)
    }
}

impl<T> From<T> for BasePosition
where
    T: Into<Position>,
{
    fn from(x: T) -> BasePosition {
        BasePosition::new(x.into(), BasePositionMode::Default)
    }
}

pub fn degrees_to_percent(x: f64, range: f64) -> f64 {
    let percent = (1.0 / (range / 2.0) * x + 1.0) / 2.0;
    clamp(percent, 0.0, 1.0)
}
