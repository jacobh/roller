use midi::Note;
use rustc_hash::FxHashMap;
use serde::Deserialize;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use crate::{
    clock::Rate,
    color::Color,
    control::midi::NoteState,
    effect::{ColorEffect, DimmerEffect, PixelEffect, PositionEffect},
    lighting_engine::{ButtonGroupInfo, ButtonInfo, LightingEvent, SceneId},
    utils::shift_remove_vec,
};

lazy_static::lazy_static! {
    static ref BUTTON_GROUP_ID_SEQ: AtomicUsize = AtomicUsize::new(0);
    static ref CLOCK_RATE_GROUP_ID: ButtonGroupId = ButtonGroupId::new();
    static ref SCENE_GROUP_ID: ButtonGroupId = ButtonGroupId::new();
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd)]
pub struct ButtonGroupId(usize);
impl ButtonGroupId {
    fn new() -> ButtonGroupId {
        let id = BUTTON_GROUP_ID_SEQ.fetch_add(1, Ordering::Relaxed);

        ButtonGroupId(id)
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
    ActivatePixelEffect(PixelEffect),
    ActivatePositionEffect(PositionEffect),
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
    pub on_action: ButtonAction,
}
impl ButtonMapping {
    pub fn into_group(self, button_type: ButtonType) -> ButtonGroup {
        ButtonGroup::new(button_type, vec![self])
    }
    pub fn into_lighting_event(
        self,
        group: ButtonGroup,
        note_state: NoteState,
        now: Instant,
    ) -> LightingEvent {
        LightingEvent::UpdateButton(group, self, note_state, now)
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ButtonGroup {
    pub id: ButtonGroupId,
    pub button_type: ButtonType,
    buttons: FxHashMap<Note, ButtonMapping>,
}
impl ButtonGroup {
    pub fn buttons(&self) -> impl Iterator<Item = &'_ ButtonMapping> {
        self.buttons.values()
    }
    pub fn new(
        button_type: ButtonType,
        buttons: impl IntoIterator<Item = ButtonMapping>,
    ) -> ButtonGroup {
        ButtonGroup {
            id: ButtonGroupId::new(),
            buttons: buttons
                .into_iter()
                .map(|button| (button.note, button))
                .collect(),
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
    group_toggle_state: GroupToggleState,
    state: AkaiPadState,
    active_group_notes: Vec<Note>,
}
impl<'a> Pad<'a> {
    fn new(mapping: PadMapping<'a>, group_toggle_state: GroupToggleState) -> Pad<'a> {
        Pad {
            mapping,
            group_toggle_state,
            active_group_notes: Vec::with_capacity(8),
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
            ButtonType::Toggle => match self.group_toggle_state {
                GroupToggleState::On(note) => {
                    if note == self.mapping.note() {
                        self.state = self.mapping.active_color();
                    } else {
                        self.state = self.mapping.inactive_color();
                    }
                }
                GroupToggleState::Off => {
                    self.state = self.mapping.inactive_color();
                }
            },
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
    Standard(&'a ButtonMapping, ButtonGroupId, ButtonType),
    Meta(&'a MetaButtonMapping),
}
impl<'a> PadMapping<'a> {
    fn note(&self) -> Note {
        match self {
            PadMapping::Standard(mapping, _, _) => mapping.note,
            PadMapping::Meta(mapping) => mapping.note,
        }
    }
    fn group_id(&self) -> Option<ButtonGroupId> {
        match self {
            PadMapping::Standard(_, group_id, _) => Some(*group_id),
            PadMapping::Meta(mapping) => match mapping.on_action {
                MetaButtonAction::TapTempo => None,
                MetaButtonAction::UpdateClockRate(_) => Some(*CLOCK_RATE_GROUP_ID),
                MetaButtonAction::ActivateScene(_) => Some(*SCENE_GROUP_ID),
            },
        }
    }
    fn button_type(&self) -> ButtonType {
        match self {
            PadMapping::Standard(_, _, button_type) => *button_type,
            PadMapping::Meta(mapping) => match mapping.on_action {
                MetaButtonAction::TapTempo => ButtonType::Flash,
                MetaButtonAction::UpdateClockRate(_) => ButtonType::Switch,
                MetaButtonAction::ActivateScene(_) => ButtonType::Switch,
            },
        }
    }
    fn active_color(&self) -> AkaiPadState {
        match self {
            PadMapping::Standard(..) => AkaiPadState::Green,
            PadMapping::Meta(_) => AkaiPadState::GreenBlink,
        }
    }
    fn inactive_color(&self) -> AkaiPadState {
        match self {
            PadMapping::Standard(..) => AkaiPadState::Yellow,
            PadMapping::Meta(_) => AkaiPadState::Yellow,
        }
    }
    fn deactivated_color(&self) -> AkaiPadState {
        match self {
            PadMapping::Standard(..) => AkaiPadState::Red,
            PadMapping::Meta(_) => AkaiPadState::Red,
        }
    }
}

impl<'a> From<(&'a ButtonGroup, &'a ButtonMapping)> for PadMapping<'a> {
    fn from((group, mapping): (&'a ButtonGroup, &'a ButtonMapping)) -> PadMapping<'a> {
        PadMapping::Standard(mapping, group.id, group.button_type)
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
}
impl<'a> PadEvent<'a> {
    pub fn new<T>(mapping: &'a T, note_state: NoteState) -> PadEvent<'a>
    where
        &'a T: Into<PadMapping<'a>>,
    {
        PadEvent {
            mapping: mapping.into(),
            note_state,
        }
    }
    pub fn new_on<T>(mapping: &'a T) -> PadEvent<'a>
    where
        &'a T: Into<PadMapping<'a>>,
    {
        PadEvent::new(mapping, NoteState::On)
    }
}

// convert from an item in the `ButtonStateMap` hashmap
impl<'a> From<(ButtonGroupInfo, ButtonInfo<'a>)> for PadEvent<'a> {
    fn from((group_info, button_info): (ButtonGroupInfo, ButtonInfo<'a>)) -> PadEvent<'a> {
        PadEvent {
            mapping: PadMapping::Standard(
                button_info.button,
                group_info.id,
                group_info.button_type,
            ),
            note_state: button_info.note_state,
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
                .and_then(|id| group_toggle_states.get(&id).copied())
                .unwrap_or_else(|| GroupToggleState::Off);

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
