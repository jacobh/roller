use roller_protocol::control::fader::{FaderControlMapping, FaderType};

use crate::lighting_engine::ControlEvent;

fn fader_type_control_event(fader_type: &FaderType, value: f64) -> ControlEvent {
    match *fader_type {
        FaderType::MasterDimmer => ControlEvent::UpdateMasterDimmer(value),
        FaderType::GroupDimmer(group_id) => ControlEvent::UpdateGroupDimmer(group_id, value),
        FaderType::DimmerEffectIntensity => ControlEvent::UpdateDimmerEffectIntensity(value),
        FaderType::ColorEffectIntensity => ControlEvent::UpdateColorEffectIntensity(value),
    }
}

pub fn fader_mapping_control_event(fader: &FaderControlMapping, value: f64) -> ControlEvent {
    fader_type_control_event(&fader.fader_type, fader.fader_curve.apply(value))
}
