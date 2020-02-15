use derive_more::Constructor;
use midi::Note;
use rustc_hash::FxHashMap;
use serde::Deserialize;
use std::time::Instant;

use crate::{
    color::Color,
    control::midi::{MidiMapping, NoteState},
    effect::{ColorModifier, DimmerEffect},
    lighting_engine::LightingEvent,
    utils::FxIndexMap,
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
    ActivateDimmerEffect(DimmerEffect),
    ActivateColorModifier(ColorModifier),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum AkaiPadState {
    Off,
    Green,
    GreenBlink,
    Red,
    RedBlink,
    Yellow,
    YellowBlink,
}
impl AkaiPadState {
    pub fn as_byte(self) -> u8 {
        match self {
            AkaiPadState::Off => 0,
            AkaiPadState::Green => 1,
            AkaiPadState::GreenBlink => 2,
            AkaiPadState::Red => 3,
            AkaiPadState::RedBlink => 4,
            AkaiPadState::Yellow => 5,
            AkaiPadState::YellowBlink => 6,
        }
    }
}

pub fn pad_states(
    midi_mapping: &MidiMapping,
    button_states: &FxIndexMap<(ButtonMapping, NoteState), (ToggleState, Instant)>,
) -> FxHashMap<Note, AkaiPadState> {
    let mut state = midi_mapping.initial_pad_states();

    let mut group_notes: FxHashMap<ButtonGroupId, Vec<Note>> = FxHashMap::default();
    for button in midi_mapping.buttons.values() {
        if let Some(group_id) = button.group_id {
            group_notes.entry(group_id).or_default().push(button.note);
        }
    }

    let mut active_group_buttons: FxHashMap<ButtonGroupId, Vec<Note>> = group_notes
        .keys()
        .map(|group_id| (*group_id, Vec::new()))
        .collect();

    for ((mapping, note_state), (toggle_state, _)) in button_states.iter() {
        match mapping.button_type {
            ButtonType::Flash => {
                // TODO groups
                state.insert(
                    mapping.note,
                    match note_state {
                        NoteState::On => AkaiPadState::Green,
                        NoteState::Off => AkaiPadState::Yellow,
                    },
                );
            }
            ButtonType::Toggle => {
                // TODO groups
                state.insert(
                    mapping.note,
                    match note_state {
                        NoteState::On => match toggle_state {
                            ToggleState::On => AkaiPadState::Green,
                            ToggleState::Off => AkaiPadState::Red,
                        },
                        NoteState::Off => match toggle_state {
                            ToggleState::On => AkaiPadState::Green,
                            ToggleState::Off => AkaiPadState::Yellow,
                        },
                    },
                );
            }
            ButtonType::Switch => match note_state {
                NoteState::On => {
                    state.insert(mapping.note, AkaiPadState::Green);

                    if let Some(group_id) = mapping.group_id {
                        let active_group_buttons = active_group_buttons.get_mut(&group_id).unwrap();
                        active_group_buttons.push(mapping.note);

                        if active_group_buttons.len() == 1 {
                            for note in group_notes[&group_id].iter() {
                                if *note != mapping.note {
                                    state.insert(*note, AkaiPadState::Red);
                                }
                            }
                        }
                    }
                }
                NoteState::Off => {
                    state.insert(mapping.note, AkaiPadState::Green);
                    if let Some(group_id) = mapping.group_id {
                        let active_group_buttons = active_group_buttons.get_mut(&group_id).unwrap();

                        let button_idx = active_group_buttons
                            .iter()
                            .position(|note| *note == mapping.note);
                        if let Some(button_idx) = button_idx {
                            active_group_buttons.remove(button_idx);
                        }

                        if active_group_buttons.is_empty() {
                            for note in group_notes[&group_id].iter() {
                                if *note != mapping.note {
                                    state.insert(*note, AkaiPadState::Yellow);
                                }
                            }
                        } else {
                            state.insert(mapping.note, AkaiPadState::Red);
                        }
                    }
                }
            },
        }
    }

    state
}
