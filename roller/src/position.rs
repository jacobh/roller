use ordered_float::OrderedFloat;
use std::ops::Add;

use crate::utils::clamp;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
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

pub fn degrees_to_percent(x: f64, range: f64) -> f64 {
    let percent = (1.0 / (range / 2.0) * x + 1.0) / 2.0;
    clamp(percent, 0.0, 1.0)
}
