use rustc_hash::FxHashMap;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use roller_protocol::{ButtonCoordinate, ButtonGridLocation, ButtonState};

use crate::{
    clock::Rate,
    color::Color,
    control::NoteState,
    effect::{ColorEffect, DimmerEffect, PixelEffect, PositionEffect},
    lighting_engine::{ButtonGroupInfo, ButtonInfo, ControlEvent, ControlMode, SceneId},
    position::BasePosition,
    project::FixtureGroupId,
    utils::shift_remove_vec,
};

lazy_static::lazy_static! {
    static ref BUTTON_GROUP_ID_SEQ: AtomicUsize = AtomicUsize::new(0);
    static ref CLOCK_RATE_GROUP_ID: ButtonGroupId = ButtonGroupId::new();
    static ref SCENE_GROUP_ID: ButtonGroupId = ButtonGroupId::new();
    static ref TOGGLE_FIXTURE_GROUP_ID: ButtonGroupId = ButtonGroupId::new();
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
    On(ButtonCoordinate),
    Off,
}
impl GroupToggleState {
    pub fn toggle(self, coord: ButtonCoordinate) -> GroupToggleState {
        if self == GroupToggleState::On(coord) {
            GroupToggleState::Off
        } else {
            GroupToggleState::On(coord)
        }
    }
    pub fn toggle_mut(&mut self, coord: ButtonCoordinate) {
        *self = self.toggle(coord);
    }
}

// Buttons are used for configurable, creative controls. activating colors, chases, etc
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ButtonAction {
    UpdateGlobalColor(Color),
    UpdateGlobalSecondaryColor(Color),
    UpdateBasePosition(BasePosition),
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
    pub coordinate: ButtonCoordinate,
    pub on_action: ButtonAction,
}
impl ButtonMapping {
    pub fn into_group(self, button_type: ButtonType) -> ButtonGroup {
        ButtonGroup::new(button_type, vec![self])
    }
}

// Meta buttons are global controls for things like tap tempo, changing page, activating a bank
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MetaButtonAction {
    EnableShiftMode,
    DisableShiftMode,
    TapTempo,
    UpdateClockRate(Rate),
    SelectScene(SceneId),
    SelectFixtureGroupControl(FixtureGroupId),
}
impl MetaButtonAction {
    pub fn control_event(&self, now: Instant) -> ControlEvent {
        match self {
            MetaButtonAction::EnableShiftMode => {
                ControlEvent::UpdateControlMode(ControlMode::Shift)
            }
            MetaButtonAction::DisableShiftMode => {
                ControlEvent::UpdateControlMode(ControlMode::Normal)
            }
            MetaButtonAction::TapTempo => ControlEvent::TapTempo(now),
            MetaButtonAction::UpdateClockRate(rate) => ControlEvent::UpdateClockRate(*rate),
            MetaButtonAction::SelectScene(scene_id) => ControlEvent::SelectScene(*scene_id),
            MetaButtonAction::SelectFixtureGroupControl(group_id) => {
                ControlEvent::SelectFixtureGroupControl(*group_id)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MetaButtonMapping {
    pub location: ButtonGridLocation,
    pub coordinate: ButtonCoordinate,
    pub on_action: MetaButtonAction,
    pub off_action: Option<MetaButtonAction>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ButtonGroup {
    pub id: ButtonGroupId,
    pub button_type: ButtonType,
    buttons: BTreeMap<ButtonCoordinate, ButtonMapping>,
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
                .map(|button| (button.coordinate, button))
                .collect(),
            button_type,
        }
    }
}

pub enum ButtonRef<'a> {
    Standard(&'a ButtonGroup, &'a ButtonMapping),
    Meta(&'a MetaButtonMapping),
}
impl<'a> ButtonRef<'a> {
    pub fn into_control_event(
        self,
        note_state: NoteState,
        now: Instant,
    ) -> Option<ControlEvent<'a>> {
        match (self, note_state) {
            (ButtonRef::Standard(group, button), _) => {
                Some(ControlEvent::UpdateButton(group, button, note_state, now))
            }
            (ButtonRef::Meta(meta_button), NoteState::On) => {
                Some(meta_button.on_action.control_event(now))
            }
            (ButtonRef::Meta(meta_button), NoteState::Off) => meta_button
                .off_action
                .as_ref()
                .map(|action| action.control_event(now)),
        }
    }
}

pub struct Pad<'a> {
    mapping: PadMapping<'a>,
    group_toggle_state: GroupToggleState,
    state: ButtonState,
    active_group_coords: Vec<ButtonCoordinate>,
}
impl<'a> Pad<'a> {
    fn new(mapping: PadMapping<'a>, group_toggle_state: GroupToggleState) -> Pad<'a> {
        Pad {
            mapping,
            group_toggle_state,
            active_group_coords: Vec::with_capacity(8),
            state: ButtonState::Inactive,
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
                        NoteState::On => ButtonState::Active,
                        NoteState::Off => ButtonState::Inactive,
                    }
                }
            }
            ButtonType::Toggle => match self.group_toggle_state {
                GroupToggleState::On(coord) => {
                    if &coord == self.mapping.coordinate() {
                        self.state = ButtonState::Active;
                    } else {
                        self.state = ButtonState::Inactive;
                    }
                }
                GroupToggleState::Off => {
                    self.state = ButtonState::Inactive;
                }
            },
            ButtonType::Switch => match event.note_state {
                NoteState::On => {
                    if event.mapping == self.mapping {
                        self.state = ButtonState::Active;
                    }

                    if group_id_match.is_match() {
                        self.active_group_coords.push(*event.mapping.coordinate());

                        if !self.active_group_coords.contains(self.mapping.coordinate()) {
                            self.state = ButtonState::Deactivated;
                        }
                    }
                }
                NoteState::Off => {
                    if group_id_match.is_match() {
                        shift_remove_vec(&mut self.active_group_coords, event.mapping.coordinate());

                        if event.mapping == self.mapping {
                            if self.active_group_coords.is_empty() {
                                // leave as green
                            } else {
                                self.state = ButtonState::Deactivated;
                            }
                        } else if self.active_group_coords.is_empty() {
                            self.state = ButtonState::Inactive;
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
    fn location(&self) -> ButtonGridLocation {
        match self {
            PadMapping::Standard(_, _, _) => ButtonGridLocation::Main,
            PadMapping::Meta(mapping) => mapping.location,
        }
    }
    fn coordinate(&self) -> &ButtonCoordinate {
        match self {
            PadMapping::Standard(mapping, _, _) => &mapping.coordinate,
            PadMapping::Meta(mapping) => &mapping.coordinate,
        }
    }
    fn group_id(&self) -> Option<ButtonGroupId> {
        match self {
            PadMapping::Standard(_, group_id, _) => Some(*group_id),
            PadMapping::Meta(mapping) => match mapping.on_action {
                MetaButtonAction::UpdateClockRate(_) => Some(*CLOCK_RATE_GROUP_ID),
                MetaButtonAction::SelectScene(_) => Some(*SCENE_GROUP_ID),
                MetaButtonAction::SelectFixtureGroupControl(_) => Some(*TOGGLE_FIXTURE_GROUP_ID),
                MetaButtonAction::TapTempo
                | MetaButtonAction::EnableShiftMode
                | MetaButtonAction::DisableShiftMode => None,
            },
        }
    }
    fn button_type(&self) -> ButtonType {
        match self {
            PadMapping::Standard(_, _, button_type) => *button_type,
            PadMapping::Meta(_) => ButtonType::Switch,
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
impl<'a> From<(ButtonGroupInfo<'a>, ButtonInfo<'a>)> for PadEvent<'a> {
    fn from((group_info, button_info): (ButtonGroupInfo, ButtonInfo<'a>)) -> PadEvent<'a> {
        PadEvent {
            mapping: PadMapping::Standard(
                button_info.button,
                group_info.group.id,
                group_info.group.button_type,
            ),
            note_state: button_info.note_state,
        }
    }
}

pub fn pad_states<'a>(
    all_pads: Vec<PadMapping<'a>>,
    group_toggle_states: &FxHashMap<ButtonGroupId, GroupToggleState>,
    pad_events: impl IntoIterator<Item = PadEvent<'a>>,
) -> FxHashMap<(ButtonGridLocation, ButtonCoordinate), ButtonState> {
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
        .map(|pad| {
            (
                (pad.mapping.location(), *pad.mapping.coordinate()),
                pad.state,
            )
        })
        .collect()
}
