use crate::{lighting_engine::LightingEvent, project::FixtureGroupId};
use midi::ControlChannel;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FaderType {
    MasterDimmer,
    GroupDimmer(FixtureGroupId),
    GlobalEffectIntensity,
}
impl FaderType {
    pub fn lighting_event(&self, value: f64) -> LightingEvent {
        match *self {
            FaderType::MasterDimmer => LightingEvent::UpdateMasterDimmer { dimmer: value },
            FaderType::GroupDimmer(group_id) => LightingEvent::UpdateGroupDimmer {
                group_id,
                dimmer: value,
            },
            FaderType::GlobalEffectIntensity => LightingEvent::UpdateGlobalEffectIntensity(value),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MidiFaderMapping {
    pub control_channel: ControlChannel,
    pub fader_type: FaderType,
}
