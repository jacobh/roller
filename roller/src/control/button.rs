use derive_more::Constructor;
use midi::Note;
use serde::Deserialize;
use std::time::Instant;

use crate::{
    color::Color, control::midi::NoteState, effect::DimmerModifier, lighting_engine::LightingEvent,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Constructor, Deserialize)]
pub struct ButtonGroupId(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ToggleState {
    On,
    Off,
}
impl ToggleState {
    pub fn toggle(self) -> ToggleState {
        match self {
            ToggleState::On => ToggleState::Off,
            ToggleState::Off => ToggleState::On,
        }
    }
}

// Buttons are used for configurable, creative controls. activating colors, chases, etc
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ButtonAction {
    UpdateGlobalColor(Color),
    ActivateDimmerModifier(DimmerModifier),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ButtonType {
    // Once enabled, this button, or a button in its group, must stay on)
    Switch,
    // Buttons that may be enabled and disabled
    Toggle,
    // Buttons that will stay enabled only while the note is held down
    Flash,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ButtonMapping {
    pub note: Note,
    pub button_type: ButtonType,
    pub group_id: Option<ButtonGroupId>,
    pub on_action: ButtonAction,
}
impl ButtonMapping {
    pub fn into_lighting_event(self, note_state: NoteState, now: Instant) -> LightingEvent {
        LightingEvent::UpdateButton(now, note_state, self)
    }
}

// Meta buttons are global controls for things like tap tempo, changing page, activating a bank
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MetaButtonAction {
    TapTempo,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MetaButtonMapping {
    pub note: Note,
    pub on_action: MetaButtonAction,
}
impl MetaButtonMapping {
    pub fn lighting_event(&self, now: Instant) -> LightingEvent {
        match self.on_action {
            MetaButtonAction::TapTempo => LightingEvent::TapTempo(now),
        }
    }
}
