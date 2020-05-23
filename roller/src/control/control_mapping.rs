use roller_protocol::FaderId;
use rustc_hash::FxHashMap;

use roller_protocol::{ButtonCoordinate, ButtonGridLocation};

use crate::{
    control::{
        button::{ButtonGroup, ButtonMapping, MetaButtonMapping, PadMapping, ButtonRef},
        fader::FaderControlMapping,
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlMapping {
    pub faders: FxHashMap<FaderId, FaderControlMapping>,
    pub button_groups: Vec<ButtonGroup>,
    pub meta_buttons: FxHashMap<(ButtonGridLocation, ButtonCoordinate), MetaButtonMapping>,
}
impl ControlMapping {
    pub fn new(
        faders: Vec<FaderControlMapping>,
        button_groups: Vec<ButtonGroup>,
        meta_buttons: Vec<MetaButtonMapping>,
    ) -> ControlMapping {
        ControlMapping {
            faders: faders
                .into_iter()
                .map(|mapping| (mapping.id, mapping))
                .collect(),
            button_groups,
            meta_buttons: meta_buttons
                .into_iter()
                .map(|mapping| ((mapping.location, mapping.coordinate), mapping))
                .collect(),
        }
    }
    fn group_buttons(&self) -> impl Iterator<Item = (&'_ ButtonGroup, &'_ ButtonMapping)> {
        self.button_groups
            .iter()
            .flat_map(|group| group.buttons().map(move |button| (group, button)))
    }
    pub fn find_button(
        &self,
        location: ButtonGridLocation,
        coordinate: ButtonCoordinate,
    ) -> Option<ButtonRef<'_>> {
        if location == ButtonGridLocation::Main {
            self.group_buttons()
                .find(|(_, button)| button.coordinate == coordinate)
                .map(|(group, button)| ButtonRef::Standard(group, button))
        } else {
            self.meta_buttons
                .get(&(location, coordinate))
                .map(|meta_button| ButtonRef::Meta(meta_button))
        }
    }
    pub fn pad_mappings(&self) -> impl Iterator<Item = PadMapping<'_>> {
        self.group_buttons()
            .map(PadMapping::from)
            .chain(self.meta_buttons.values().map(PadMapping::from))
    }
}