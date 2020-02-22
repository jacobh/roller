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

pub struct Pad<'a, T>
where
    T: PadMapping,
{
    mapping: &'a T,
    state: AkaiPadState,
    active_group_notes: Vec<Note>,
}
impl<'a, T> Pad<'a, T>
where
    T: PadMapping,
{
    fn new(mapping: &'a T) -> Pad<'a, T> {
        Pad {
            mapping,
            state: AkaiPadState::Yellow,
            active_group_notes: Vec::new(),
        }
    }
    fn apply_event(&mut self, event: &PadEvent<'a, T>) {
        let group_id_match =
            ButtonGroupIdMatch::match_(event.mapping.group_id(), self.mapping.group_id());
        // If this event isn't for the current mapping or a mapping in this group, short circuit
        if event.mapping != self.mapping && !group_id_match.is_match() {
            return;
        }

        match self.mapping.button_type() {
            ButtonType::Flash => {
                if event.mapping == self.mapping {
                    self.state = match event.note_state {
                        NoteState::On => AkaiPadState::Green,
                        NoteState::Off => AkaiPadState::Yellow,
                    }
                }
            }
            ButtonType::Toggle => {
                // TODO groups
                if event.mapping == self.mapping {
                    self.state = match event.note_state {
                        NoteState::On => match event.toggle_state {
                            ToggleState::On => AkaiPadState::Green,
                            ToggleState::Off => AkaiPadState::Red,
                        },
                        NoteState::Off => match event.toggle_state {
                            ToggleState::On => AkaiPadState::Green,
                            ToggleState::Off => AkaiPadState::Yellow,
                        },
                    }
                }
            }
            ButtonType::Switch => match event.note_state {
                NoteState::On => {
                    if event.mapping == self.mapping {
                        self.state = AkaiPadState::Green;
                    }

                    if group_id_match.is_match() {
                        self.active_group_notes.push(event.mapping.note());

                        if !self.active_group_notes.contains(&self.mapping.note()) {
                            self.state = AkaiPadState::Red;
                        }
                    }
                }
                NoteState::Off => {
                    // TODO not required?
                    if event.mapping == self.mapping {
                        self.state = AkaiPadState::Green;
                    }

                    if group_id_match.is_match() {
                        let pad_idx = self
                            .active_group_notes
                            .iter()
                            .position(|note| *note == event.mapping.note());

                        if let Some(pad_idx) = pad_idx {
                            self.active_group_notes.remove(pad_idx);
                        }

                        if self.active_group_notes.is_empty() && event.mapping != self.mapping {
                            self.state = AkaiPadState::Yellow;
                        }
                    }
                }
            },
        }
    }
}

enum ButtonGroupIdMatch {
    MatchingGroupId(ButtonGroupId),
    NoGroupId,
    ConflictingGroupIds,
}
impl ButtonGroupIdMatch {
    fn match_(a: Option<ButtonGroupId>, b: Option<ButtonGroupId>) -> ButtonGroupIdMatch {
        match (a, b) {
            (Some(a), Some(b)) => {
                if a == b {
                    ButtonGroupIdMatch::MatchingGroupId(a)
                } else {
                    ButtonGroupIdMatch::ConflictingGroupIds
                }
            }
            (Some(_), None) | (None, Some(_)) => ButtonGroupIdMatch::ConflictingGroupIds,
            (None, None) => ButtonGroupIdMatch::NoGroupId,
        }
    }
    fn is_match(&self) -> bool {
        match self {
            ButtonGroupIdMatch::MatchingGroupId(_) => true,
            _ => false,
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
    let mut state: Vec<_> = all_pads.iter().map(|mapping| Pad::new(*mapping)).collect();

    for event in pad_events {
        for pad in state.iter_mut() {
            pad.apply_event(&event);
        }
    }

    state
        .into_iter()
        .map(|pad| (pad.mapping.note(), pad.state))
        .collect()
}
