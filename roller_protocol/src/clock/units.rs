use derive_more::{From, Into};
use ordered_float::OrderedFloat;
use serde::{Serialize, Deserialize};
use std::iter::Sum;
use std::ops::{Add, Mul, Sub};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, From, Into, Serialize, Deserialize)]
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, From, Into, Serialize, Deserialize)]
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
impl Mul for Rate {
    type Output = Rate;
    fn mul(self, other: Rate) -> Rate {
        Rate::new(self.0.into_inner() * other.0.into_inner())
    }
}
