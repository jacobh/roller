#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FaderType {
    MasterDimmer,
    GroupDimmer { group_id: usize },
    GlobalEffectIntensity,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MidiFaderMapping {
    pub control_channel: u8,
    pub fader_type: FaderType,
}
