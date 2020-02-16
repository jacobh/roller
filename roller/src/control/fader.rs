use crate::{effect::sigmoid, lighting_engine::LightingEvent, project::FixtureGroupId};
use midi::ControlChannel;
use ordered_float::OrderedFloat;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FaderType {
    MasterDimmer,
    GroupDimmer(FixtureGroupId),
    DimmerEffectIntensity,
    ColorEffectIntensity,
}
impl FaderType {
    fn lighting_event(&self, value: f64) -> LightingEvent {
        match *self {
            FaderType::MasterDimmer => LightingEvent::UpdateMasterDimmer { dimmer: value },
            FaderType::GroupDimmer(group_id) => LightingEvent::UpdateGroupDimmer {
                group_id,
                dimmer: value,
            },
            FaderType::DimmerEffectIntensity => LightingEvent::UpdateDimmerEffectIntensity(value),
            FaderType::ColorEffectIntensity => LightingEvent::UpdateColorEffectIntensity(value),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MidiFaderMapping {
    pub control_channel: ControlChannel,
    pub fader_type: FaderType,
    pub fader_curve: FaderCurve,
}
impl MidiFaderMapping {
    pub fn lighting_event(&self, value: f64) -> LightingEvent {
        self.fader_type
            .lighting_event(self.fader_curve.apply(value))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
