use derive_more::Constructor;
use midi::Note;
use ordered_float::OrderedFloat;
use rustc_hash::FxHashMap;
use serde::Deserialize;
use std::time::Instant;

use crate::{
    color::Color,
    control::midi::NoteState,
    effect::{ColorEffect, DimmerEffect},
    lighting_engine::{LightingEvent, SceneId},
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
    ActivateColorEffect(ColorEffect),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    UpdateGlobalSpeedMultiplier(OrderedFloat<f64>),
    ActivateScene(SceneId),
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
            MetaButtonAction::UpdateGlobalSpeedMultiplier(multiplier) => {
                LightingEvent::UpdateGlobalSpeedMultiplier(multiplier.into_inner())
            }
            MetaButtonAction::ActivateScene(scene_id) => LightingEvent::ActivateScene(scene_id),
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

pub trait PadMapping: std::hash::Hash + Eq {
    fn note(&self) -> Note;
    fn group_id(&self) -> Option<ButtonGroupId>;
    fn button_type(&self) -> ButtonType;
}

impl PadMapping for ButtonMapping {
    fn note(&self) -> Note {
        self.note
    }
    fn group_id(&self) -> Option<ButtonGroupId> {
        self.group_id
    }
    fn button_type(&self) -> ButtonType {
        self.button_type
    }
}

impl PadMapping for MetaButtonMapping {
    fn note(&self) -> Note {
        self.note
    }
    fn group_id(&self) -> Option<ButtonGroupId> {
        match self.on_action {
            MetaButtonAction::TapTempo => None,
            MetaButtonAction::UpdateGlobalSpeedMultiplier(_) => Some(ButtonGroupId::new(1)),
            MetaButtonAction::ActivateScene(_) => Some(ButtonGroupId::new(2)),
        }
    }
    fn button_type(&self) -> ButtonType {
        match self.on_action {
            MetaButtonAction::TapTempo => ButtonType::Flash,
            MetaButtonAction::UpdateGlobalSpeedMultiplier(_) => ButtonType::Switch,
            MetaButtonAction::ActivateScene(_) => ButtonType::Switch,
        }
    }
}

pub struct PadEvent<'a, T>
where
    T: PadMapping,
{
    mapping: &'a T,
    note_state: NoteState,
    toggle_state: ToggleState,
}

// convert from an item in the `ButtonStateMap` hashmap
impl<'a> From<(&'a (ButtonMapping, NoteState), &'a (ToggleState, Instant))>
    for PadEvent<'a, ButtonMapping>
{
    fn from(
        ((mapping, note_state), (toggle_state, _)): (
            &'a (ButtonMapping, NoteState),
            &'a (ToggleState, Instant),
        ),
    ) -> PadEvent<'a, ButtonMapping> {
        PadEvent {
            mapping,
            note_state: *note_state,
            toggle_state: *toggle_state,
        }
    }
}

pub fn pad_states<'a, T>(
    all_pads: Vec<&T>,
    pad_events: impl Iterator<Item = PadEvent<'a, T>>,
) -> FxHashMap<Note, AkaiPadState>
where
    T: 'a + PadMapping,
{
    let mut state: FxHashMap<_, _> = all_pads
        .iter()
        .map(|mapping| (mapping.note(), AkaiPadState::Yellow))
        .collect();

    let mut group_notes: FxHashMap<ButtonGroupId, Vec<Note>> = FxHashMap::default();
    for pad in all_pads.iter() {
        if let Some(group_id) = pad.group_id() {
            group_notes.entry(group_id).or_default().push(pad.note());
        }
    }

    let mut active_group_pads: FxHashMap<ButtonGroupId, Vec<Note>> = group_notes
        .keys()
        .map(|group_id| (*group_id, Vec::new()))
        .collect();

    for PadEvent {
        mapping,
        note_state,
        toggle_state,
    } in pad_events
    {
        match mapping.button_type() {
            ButtonType::Flash => {
                // TODO groups
                state.insert(
                    mapping.note(),
                    match note_state {
                        NoteState::On => AkaiPadState::Green,
                        NoteState::Off => AkaiPadState::Yellow,
                    },
                );
            }
            ButtonType::Toggle => {
                // TODO groups
                state.insert(
                    mapping.note(),
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
                    state.insert(mapping.note(), AkaiPadState::Green);

                    if let Some(group_id) = mapping.group_id() {
                        let active_group_pads = active_group_pads.get_mut(&group_id).unwrap();
                        active_group_pads.push(mapping.note());

                        if active_group_pads.len() == 1 {
                            for note in group_notes[&group_id].iter() {
                                if *note != mapping.note() {
                                    state.insert(*note, AkaiPadState::Red);
                                }
                            }
                        }
                    }
                }
                NoteState::Off => {
                    state.insert(mapping.note(), AkaiPadState::Green);
                    if let Some(group_id) = mapping.group_id() {
                        let active_group_pads = active_group_pads.get_mut(&group_id).unwrap();

                        let pad_idx = active_group_pads
                            .iter()
                            .position(|note| *note == mapping.note());
                        if let Some(pad_idx) = pad_idx {
                            active_group_pads.remove(pad_idx);
                        }

                        if active_group_pads.is_empty() {
                            for note in group_notes[&group_id].iter() {
                                if *note != mapping.note() {
                                    state.insert(*note, AkaiPadState::Yellow);
                                }
                            }
                        } else {
                            state.insert(mapping.note(), AkaiPadState::Red);
                        }
                    }
                }
            },
        }
    }

    state
}
