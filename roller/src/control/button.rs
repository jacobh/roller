use derive_more::Constructor;
use midi::Note;
use rustc_hash::FxHashMap;
use serde::Deserialize;
use std::time::Instant;

use crate::{
    clock::Rate,
    color::Color,
    control::midi::NoteState,
    effect::{ColorEffect, DimmerEffect},
    lighting_engine::{LightingEvent, SceneId},
    utils::shift_remove_vec,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Constructor, Deserialize)]
pub struct ButtonGroupId(usize);
impl ButtonGroupId {
    fn new_random() -> ButtonGroupId {
        ButtonGroupId(rand::random())
    }
}

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

/// An enum tracking the toggle state of a button group
/// When toggled with the same note, it will turn off, when toggled with a
/// different note, it will stay on with the internal note updated
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GroupToggleState {
    On(Note),
    Off,
}
impl GroupToggleState {
    pub fn toggle(self, note: Note) -> GroupToggleState {
        if self == GroupToggleState::On(note) {
            GroupToggleState::Off
        } else {
            GroupToggleState::On(note)
        }
    }
    pub fn toggle_mut(&mut self, note: Note) {
        *self = self.toggle(note);
    }
}

// Buttons are used for configurable, creative controls. activating colors, chases, etc
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ButtonAction {
    UpdateGlobalColor(Color),
    UpdateGlobalSecondaryColor(Color),
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
    UpdateClockRate(Rate),
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
            MetaButtonAction::UpdateClockRate(rate) => LightingEvent::UpdateClockRate(rate),
            MetaButtonAction::ActivateScene(scene_id) => LightingEvent::ActivateScene(scene_id),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ButtonGroup {
    id: ButtonGroupId,
    button_type: ButtonType,
    buttons: Vec<ButtonMapping>,
}
impl ButtonGroup {
    fn new(buttons: Vec<ButtonMapping>, button_type: ButtonType) -> ButtonGroup {
        ButtonGroup {
            id: ButtonGroupId::new_random(),
            buttons,
            button_type,
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

pub struct Pad<'a> {
    mapping: PadMapping<'a>,
    group_toggle_state: Option<GroupToggleState>,
    state: AkaiPadState,
    active_group_notes: Vec<Note>,
}
impl<'a> Pad<'a> {
    fn new(mapping: PadMapping<'a>, group_toggle_state: Option<GroupToggleState>) -> Pad<'a> {
        let active_group_notes = if mapping.group_id().is_some() {
            Vec::with_capacity(8)
        } else {
            Vec::new()
        };
        Pad {
            mapping,
            group_toggle_state,
            active_group_notes,
            state: AkaiPadState::Yellow,
        }
    }
    fn apply_event(&mut self, event: &PadEvent<'a>) {
        let group_id_match =
            ButtonGroupIdMatch::match_(event.mapping.group_id(), self.mapping.group_id());
        // If this event isn't for the current mapping or a mapping in this group, short circuit
        if event.mapping != self.mapping && !group_id_match.is_match() {
            return;
        }

        match event.mapping.button_type() {
            ButtonType::Flash => {
                if event.mapping == self.mapping {
                    self.state = match event.note_state {
                        NoteState::On => self.mapping.active_color(),
                        NoteState::Off => self.mapping.inactive_color(),
                    }
                }
            }
            ButtonType::Toggle => {
                if event.mapping == self.mapping {
                    self.state = match event.note_state {
                        NoteState::On => match event.toggle_state {
                            ToggleState::On => self.mapping.active_color(),
                            ToggleState::Off => self.mapping.deactivated_color(),
                        },
                        NoteState::Off => match event.toggle_state {
                            ToggleState::On => self.mapping.active_color(),
                            ToggleState::Off => self.mapping.inactive_color(),
                        },
                    }
                }

                match self.group_toggle_state {
                    Some(GroupToggleState::On(note)) => {
                        if note == self.mapping.note() {
                            self.state = self.mapping.active_color();
                        } else {
                            self.state = self.mapping.inactive_color();
                        }
                    }
                    Some(GroupToggleState::Off) => {
                        self.state = self.mapping.inactive_color();
                    }
                    None => {}
                }
            }
            ButtonType::Switch => match event.note_state {
                NoteState::On => {
                    if event.mapping == self.mapping {
                        self.state = self.mapping.active_color();
                    }

                    if group_id_match.is_match() {
                        self.active_group_notes.push(event.mapping.note());

                        if !self.active_group_notes.contains(&self.mapping.note()) {
                            self.state = self.mapping.deactivated_color();
                        }
                    }
                }
                NoteState::Off => {
                    if group_id_match.is_match() {
                        shift_remove_vec(&mut self.active_group_notes, &event.mapping.note());

                        if event.mapping == self.mapping {
                            if self.active_group_notes.is_empty() {
                                // leave as green
                            } else {
                                self.state = self.mapping.deactivated_color();
                            }
                        } else if self.active_group_notes.is_empty() {
                            self.state = self.mapping.inactive_color();
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PadMapping<'a> {
    Standard(&'a ButtonMapping),
    Meta(&'a MetaButtonMapping),
}
impl<'a> PadMapping<'a> {
    fn note(&self) -> Note {
        match self {
            PadMapping::Standard(mapping) => mapping.note,
            PadMapping::Meta(mapping) => mapping.note,
        }
    }
    fn group_id(&self) -> Option<ButtonGroupId> {
        match self {
            PadMapping::Standard(mapping) => mapping.group_id,
            PadMapping::Meta(mapping) => match mapping.on_action {
                MetaButtonAction::TapTempo => None,
                MetaButtonAction::UpdateClockRate(_) => Some(ButtonGroupId::new(100_001)),
                MetaButtonAction::ActivateScene(_) => Some(ButtonGroupId::new(100_002)),
            },
        }
    }
    fn button_type(&self) -> ButtonType {
        match self {
            PadMapping::Standard(mapping) => mapping.button_type,
            PadMapping::Meta(mapping) => match mapping.on_action {
                MetaButtonAction::TapTempo => ButtonType::Flash,
                MetaButtonAction::UpdateClockRate(_) => ButtonType::Switch,
                MetaButtonAction::ActivateScene(_) => ButtonType::Switch,
            },
        }
    }
    fn active_color(&self) -> AkaiPadState {
        match self {
            PadMapping::Standard(_) => AkaiPadState::Green,
            PadMapping::Meta(_) => AkaiPadState::GreenBlink,
        }
    }
    fn inactive_color(&self) -> AkaiPadState {
        match self {
            PadMapping::Standard(_) => AkaiPadState::Yellow,
            PadMapping::Meta(_) => AkaiPadState::Yellow,
        }
    }
    fn deactivated_color(&self) -> AkaiPadState {
        match self {
            PadMapping::Standard(_) => AkaiPadState::Red,
            PadMapping::Meta(_) => AkaiPadState::Red,
        }
    }
}

impl<'a> From<&'a ButtonMapping> for PadMapping<'a> {
    fn from(mapping: &'a ButtonMapping) -> PadMapping<'a> {
        PadMapping::Standard(mapping)
    }
}
impl<'a> From<&'a MetaButtonMapping> for PadMapping<'a> {
    fn from(mapping: &'a MetaButtonMapping) -> PadMapping<'a> {
        PadMapping::Meta(mapping)
    }
}

pub struct PadEvent<'a> {
    mapping: PadMapping<'a>,
    note_state: NoteState,
    toggle_state: ToggleState,
}
impl<'a> PadEvent<'a> {
    pub fn new<T>(mapping: &'a T, note_state: NoteState, toggle_state: ToggleState) -> PadEvent<'a>
    where
        &'a T: Into<PadMapping<'a>>,
    {
        PadEvent {
            mapping: mapping.into(),
            note_state,
            toggle_state,
        }
    }
    pub fn new_on<T>(mapping: &'a T) -> PadEvent<'a>
    where
        &'a T: Into<PadMapping<'a>>,
    {
        PadEvent::new(mapping, NoteState::On, ToggleState::On)
    }
}

// convert from an item in the `ButtonStateMap` hashmap
impl<'a>
    From<(
        &'a (ButtonMapping, NoteState),
        &'a (ToggleState, Instant, Rate),
    )> for PadEvent<'a>
{
    fn from(
        ((mapping, note_state), (toggle_state, _, _)): (
            &'a (ButtonMapping, NoteState),
            &'a (ToggleState, Instant, Rate),
        ),
    ) -> PadEvent<'a> {
        PadEvent {
            mapping: PadMapping::Standard(mapping),
            note_state: *note_state,
            toggle_state: *toggle_state,
        }
    }
}

pub fn pad_states<'a>(
    all_pads: Vec<PadMapping<'a>>,
    group_toggle_states: &FxHashMap<ButtonGroupId, GroupToggleState>,
    pad_events: impl IntoIterator<Item = PadEvent<'a>>,
) -> FxHashMap<Note, AkaiPadState> {
    let mut state: Vec<_> = all_pads
        .into_iter()
        .map(|mapping| {
            let toggle_state = mapping
                .group_id()
                .and_then(|id| group_toggle_states.get(&id).copied());

            Pad::new(mapping, toggle_state)
        })
        .collect();

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
