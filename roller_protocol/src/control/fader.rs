use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use crate::{control::FaderId, effect::sigmoid, fixture::FixtureGroupId};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FaderType {
    MasterDimmer,
    GroupDimmer(FixtureGroupId),
    DimmerEffectIntensity,
    ColorEffectIntensity,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FaderControlMapping {
    pub id: FaderId,
    pub fader_type: FaderType,
    pub fader_curve: FaderCurve,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FaderCurve {
    Linear,
    Sigmoid(OrderedFloat<f64>),
    Root(OrderedFloat<f64>),
}
impl FaderCurve {
    pub fn linear() -> FaderCurve {
        FaderCurve::Linear
    }
    // tilts above 1.0 will bias towards 0.0 and 1.0
    // tilts below 1.0 will bias towards 0.5
    pub fn sigmoid(tilt: f64) -> FaderCurve {
        FaderCurve::Sigmoid(tilt.into())
    }
    // roots above 1.0 bias towards 1.0
    // roots below 1.0 bias towards 0.0
    pub fn root(n: f64) -> FaderCurve {
        FaderCurve::Root(n.into())
    }
    pub fn apply(&self, x: f64) -> f64 {
        match self {
            FaderCurve::Linear => x,
            FaderCurve::Sigmoid(tilt) => sigmoid(x, tilt.into_inner()),
            FaderCurve::Root(n) => f64::powf(x, 1.0 / n.into_inner()),
        }
    }
}
