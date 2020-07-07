use rustc_hash::FxHashMap;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use roller_protocol::{
    fixture::FixtureGroupId, position::BasePosition, ButtonCoordinate, ButtonGridLocation,
    ButtonState, InputEvent,
};

use crate::{
    clock::Rate,
    color::Color,
    control::{control_mapping::ControlMapping, NoteState},
    effect::{ColorEffect, DimmerEffect, PixelEffect, PositionEffect},
    lighting_engine::{ControlEvent, ControlMode, SceneId},
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
    pub label: String,
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

#[derive(Debug, Clone)]
pub struct ButtonGroup {
    id: ButtonGroupId,
    pub button_type: ButtonType,
    pub buttons: Vec<ButtonMapping>,
}
impl ButtonGroup {
    pub fn new(
        button_type: ButtonType,
        buttons: impl IntoIterator<Item = ButtonMapping>,
    ) -> ButtonGroup {
        ButtonGroup {
            id: ButtonGroupId::new(),
            buttons: buttons.into_iter().collect(),
            button_type,
        }
    }
    pub fn id(&self) -> ButtonGroupId {
        self.id
    }
    pub fn iter(&self) -> impl Iterator<Item = (&'_ ButtonGroup, &'_ ButtonMapping)> {
        self.buttons.iter().map(move |button| (self, button))
    }
    pub fn button_refs(&self) -> impl Iterator<Item = ButtonRef<'_>> {
        self.iter().map(ButtonRef::from)
    }
    pub fn find_button(
        &self,
        location: &ButtonGridLocation,
        coordinate: &ButtonCoordinate,
    ) -> Option<&'_ ButtonMapping> {
        if location == &ButtonGridLocation::Main {
            self.buttons
                .iter()
                .find(|button| &button.coordinate == coordinate)
        } else {
            None
        }
    }
    pub fn find_button_ref(
        &self,
        location: &ButtonGridLocation,
        coordinate: &ButtonCoordinate,
    ) -> Option<ButtonRef<'_>> {
        self.find_button(location, coordinate)
            .map(move |button| ButtonRef::Standard(self, button))
    }
}
impl PartialEq for ButtonGroup {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for ButtonGroup {}
impl Hash for ButtonGroup {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ButtonRef<'a> {
    Standard(&'a ButtonGroup, &'a ButtonMapping),
    Meta(&'a MetaButtonMapping),
}
impl<'a> ButtonRef<'a> {
    pub fn label(&self) -> &'a str {
        match self {
            ButtonRef::Standard(_, button) => &button.label,
            ButtonRef::Meta(_) => "",
        }
    }
    pub fn location(&self) -> ButtonGridLocation {
        match self {
            ButtonRef::Standard(_, _) => ButtonGridLocation::Main,
            ButtonRef::Meta(mapping) => mapping.location,
        }
    }
    pub fn coordinate(&self) -> &ButtonCoordinate {
        match self {
            ButtonRef::Standard(_, mapping) => &mapping.coordinate,
            ButtonRef::Meta(mapping) => &mapping.coordinate,
        }
    }
    fn button_type(&self) -> ButtonType {
        match self {
            ButtonRef::Standard(group, _) => group.button_type,
            ButtonRef::Meta(_) => ButtonType::Switch,
        }
    }
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

impl<'a> From<(&'a ButtonGroup, &'a ButtonMapping)> for ButtonRef<'a> {
    fn from((group, mapping): (&'a ButtonGroup, &'a ButtonMapping)) -> ButtonRef<'a> {
        ButtonRef::Standard(group, mapping)
    }
}
impl<'a> From<&'a MetaButtonMapping> for ButtonRef<'a> {
    fn from(mapping: &'a MetaButtonMapping) -> ButtonRef<'a> {
        ButtonRef::Meta(mapping)
    }
}

enum PadAction {
    Pressed,
    Released,
    SiblingPressed,
    SiblingReleased,
}

pub struct Pad<'a> {
    mapping: ButtonRef<'a>,
    state: ButtonState,
}
impl<'a> Pad<'a> {
    fn new(mapping: ButtonRef<'a>) -> Pad<'a> {
        Pad {
            mapping,
            state: ButtonState::Inactive,
        }
    }
    fn apply_event(
        &mut self,
        action: PadAction,
        group_toggle_state: &GroupToggleState,
        active_group_buttons: &[ButtonRef<'_>],
    ) {
        match self.mapping.button_type() {
            ButtonType::Flash => {
                self.state = match action {
                    PadAction::Pressed => ButtonState::Active,
                    _ => ButtonState::Inactive,
                };
            }
            ButtonType::Toggle => match group_toggle_state {
                GroupToggleState::On(coord) => {
                    if coord == self.mapping.coordinate() {
                        self.state = ButtonState::Active;
                    } else {
                        self.state = ButtonState::Inactive;
                    }
                }
                GroupToggleState::Off => {
                    self.state = ButtonState::Inactive;
                }
            },
            ButtonType::Switch => match action {
                PadAction::Pressed => {
                    self.state = ButtonState::Active;
                }
                PadAction::SiblingPressed => {
                    if !active_group_buttons.contains(&self.mapping) {
                        self.state = ButtonState::Deactivated;
                    }
                }
                PadAction::Released => {
                    if active_group_buttons.is_empty() {
                        // leave as green
                    } else {
                        self.state = ButtonState::Deactivated;
                    }
                }
                PadAction::SiblingReleased => {
                    if active_group_buttons.is_empty() {
                        self.state = ButtonState::Inactive;
                    } else {
                        // do nothing
                    }
                }
            },
        }
    }
}

struct PadGroup<'a> {
    group: &'a ButtonGroup,
    active_buttons: Vec<ButtonRef<'a>>,
    toggle_state: GroupToggleState,
    pads: Vec<Pad<'a>>,
}
impl<'a> PadGroup<'a> {
    fn new(group: &'a ButtonGroup, toggle_state: GroupToggleState) -> PadGroup<'a> {
        PadGroup {
            group,
            toggle_state,
            pads: group.button_refs().map(Pad::new).collect(),
            active_buttons: Vec::with_capacity(group.buttons.len()),
        }
    }
    fn apply_event(&mut self, event: &InputEvent) {
        match event {
            InputEvent::ButtonPressed(location, coordinate) => {
                if let Some(button_ref) = self.group.find_button_ref(location, coordinate) {
                    self.active_buttons.push(button_ref);

                    for pad in self.pads.iter_mut() {
                        let pad_action = if button_ref == pad.mapping {
                            PadAction::Pressed
                        } else {
                            PadAction::SiblingPressed
                        };
                        pad.apply_event(pad_action, &self.toggle_state, &self.active_buttons);
                    }
                }
            }
            InputEvent::ButtonReleased(location, coordinate) => {
                if let Some(button_ref) = self.group.find_button_ref(location, coordinate) {
                    shift_remove_vec(&mut self.active_buttons, &button_ref);

                    for pad in self.pads.iter_mut() {
                        let pad_action = if button_ref == pad.mapping {
                            PadAction::Released
                        } else {
                            PadAction::SiblingReleased
                        };
                        pad.apply_event(pad_action, &self.toggle_state, &self.active_buttons);
                    }
                }
            }
            InputEvent::FaderUpdated(_, _) => {}
        }
    }
}

pub fn pad_states<'a>(
    control_mapping: &'a ControlMapping,
    group_toggle_states: &FxHashMap<ButtonGroupId, GroupToggleState>,
    input_events: impl IntoIterator<Item = InputEvent>,
) -> FxHashMap<ButtonRef<'a>, ButtonState> {
    let mut state: Vec<PadGroup<'_>> = control_mapping
        .button_groups
        .iter()
        .map(|button_group| {
            let toggle_state = group_toggle_states
                .get(&button_group.id)
                .copied()
                .unwrap_or(GroupToggleState::Off);

            PadGroup::new(button_group, toggle_state)
        })
        .collect();

    let mut meta_pads: Vec<Pad<'_>> = control_mapping
        .meta_buttons
        .values()
        .map(ButtonRef::from)
        .map(|button_ref| Pad::new(button_ref))
        .collect();

    for event in input_events {
        for pad_group in state.iter_mut() {
            pad_group.apply_event(&event);
        }
        for pad in meta_pads.iter_mut() {
            match event {
                InputEvent::ButtonPressed(location, coordinate) => {
                    if (location, coordinate) == (pad.mapping.location(), *pad.mapping.coordinate())
                    {
                        pad.apply_event(PadAction::Pressed, &GroupToggleState::Off, &[]);
                    }
                }
                InputEvent::ButtonReleased(location, coordinate) => {
                    if (location, coordinate) == (pad.mapping.location(), *pad.mapping.coordinate())
                    {
                        pad.apply_event(PadAction::Released, &GroupToggleState::Off, &[]);
                    }
                }
                InputEvent::FaderUpdated(_, _) => {}
            }
        }
    }

    state
        .into_iter()
        .flat_map(|group| group.pads.into_iter())
        .chain(meta_pads.into_iter())
        .map(|pad| (pad.mapping, pad.state))
        .collect()
}
