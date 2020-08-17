use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use roller_protocol::{control::FaderId, effect::sigmoid, fixture::FixtureGroupId};

use crate::lighting_engine::ControlEvent;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FaderType {
    MasterDimmer,
    GroupDimmer(FixtureGroupId),
    DimmerEffectIntensity,
    ColorEffectIntensity,
}


fn fader_type_control_event(fader_type: &FaderType, value: f64) -> ControlEvent {
    match *fader_type {
        FaderType::MasterDimmer => ControlEvent::UpdateMasterDimmer(value),
        FaderType::GroupDimmer(group_id) => ControlEvent::UpdateGroupDimmer(group_id, value),
        FaderType::DimmerEffectIntensity => ControlEvent::UpdateDimmerEffectIntensity(value),
        FaderType::ColorEffectIntensity => ControlEvent::UpdateColorEffectIntensity(value),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FaderControlMapping {
    pub id: FaderId,
    pub fader_type: FaderType,
    pub fader_curve: FaderCurve,
}

pub fn fader_mapping_control_event(fader: &FaderControlMapping, value: f64) -> ControlEvent {
    fader_type_control_event(&fader.fader_type, fader.fader_curve.apply(value))
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
