use rustc_hash::{FxHashMap, FxHashSet};
use std::time::Instant;

use roller_protocol::{
    clock::Rate,
    color::Color,
    control::{ButtonCoordinate, NoteState},
    effect::{ColorEffect, DimmerEffect, PixelEffect, PositionEffect},
    fixture::FixtureGroupId,
    position::BasePosition,
};

use crate::{
    control::button::{
        ButtonAction, ButtonGroup, ButtonGroupId, ButtonMapping, ButtonType, GroupToggleState,
    },
    lighting_engine::FixtureGroupValue,
    utils::{shift_remove_vec, FxIndexMap},
};

// This is just for the case where no buttons have been activated yet
lazy_static::lazy_static! {
    pub static ref EMPTY_FIXTURE_GROUP_STATE: FixtureGroupState = FixtureGroupState::default();
    pub static ref EMPTY_SCENE_STATE: SceneState = SceneState::default();
}

pub type ButtonStateMap = FxIndexMap<(ButtonMapping, NoteState), ButtonStateValue>;
pub type ButtonStateValue = (Instant, Rate);

#[derive(Default)]
pub struct SceneState {
    // contains base effect states, for all fixtures
    pub base: FixtureGroupState,
    // Contains states for effects enabled for specific groups. These take
    // precedence over any effects set in the `default` state
    pub fixture_groups: FxHashMap<FixtureGroupId, FixtureGroupState>,

    pub dimmer_effect_intensity: f64,
    pub color_effect_intensity: f64,
}
impl SceneState {
    pub fn fixture_group_state(
        &self,
        fixture_group_id: Option<FixtureGroupId>,
    ) -> &FixtureGroupState {
        if let Some(group_id) = fixture_group_id {
            self.fixture_groups
                .get(&group_id)
                .unwrap_or_else(|| &*EMPTY_FIXTURE_GROUP_STATE)
        } else {
            &self.base
        }
    }
    pub fn fixture_group_state_mut(
        &mut self,
        fixture_group_id: Option<FixtureGroupId>,
    ) -> &mut FixtureGroupState {
        if let Some(group_id) = fixture_group_id {
            self.fixture_groups.entry(group_id).or_default()
        } else {
            &mut self.base
        }
    }
    pub fn fixture_group_values(
        &self,
    ) -> (
        FixtureGroupValue<'_>,
        FxHashMap<FixtureGroupId, FixtureGroupValue<'_>>,
    ) {
        let mut base_values = self.base.fixture_group_value();

        // If fixture groups don't have intensities set, apply the default setting from the scene
        if base_values.dimmer_effect_intensity == None {
            base_values.dimmer_effect_intensity = Some(self.dimmer_effect_intensity);
        }
        if base_values.color_effect_intensity == None {
            base_values.color_effect_intensity = Some(self.color_effect_intensity);
        }

        let group_values = self
            .fixture_groups
            .iter()
            .map(|(id, state)| {
                let values = state.fixture_group_value();
                (*id, values.merge(&base_values))
            })
            .collect();

        (base_values, group_values)
    }
}

#[derive(Debug)]
pub struct FixtureGroupState {
    pub dimmer: f64,
    pub clock_rate: Rate,
    pub button_states: ButtonStates,
}
impl FixtureGroupState {
    pub fn fixture_group_value(&self) -> FixtureGroupValue<'_> {
        let buttons = &self.button_states;

        FixtureGroupValue {
            dimmer: self.dimmer,
            dimmer_effect_intensity: None,
            color_effect_intensity: None,
            clock_rate: self.clock_rate,
            global_color: buttons.global_color(),
            secondary_color: buttons.secondary_color(),
            active_dimmer_effects: buttons.active_dimmer_effects(),
            active_color_effects: buttons.active_color_effects(),
            active_pixel_effects: buttons.active_pixel_effects(),
            active_position_effects: buttons.active_position_effects(),
            base_position: buttons.base_position(),
        }
    }
}
impl Default for FixtureGroupState {
    fn default() -> FixtureGroupState {
        FixtureGroupState {
            dimmer: 1.0,
            clock_rate: Rate::default(),
            button_states: ButtonStates::default(),
        }
    }
}

type GroupStatesValue = (GroupToggleState, ButtonStateMap);
#[derive(Debug, Default)]
pub struct ButtonStates {
    group_states: FxHashMap<ButtonGroup, GroupStatesValue>,
}
impl ButtonStates {
    fn iter_groups(
        &self,
    ) -> impl Iterator<Item = (&ButtonGroup, GroupToggleState, &ButtonStateMap)> {
        self.group_states
            .iter()
            .map(|(group, (toggle_state, states))| (group, *toggle_state, states))
    }
    // Takes a button group and returns an iterator of `Info` summaries
    pub fn iter_info(&self) -> impl Iterator<Item = (ButtonGroupInfo, ButtonInfo<'_>)> {
        self.iter_groups()
            .flat_map(|(group, toggle_state, states)| {
                states
                    .iter()
                    .map(move |((button, note_state), (triggered_at, effect_rate))| {
                        (
                            ButtonGroupInfo {
                                group,
                                toggle_state: toggle_state,
                            },
                            ButtonInfo {
                                button: button,
                                note_state: *note_state,
                                triggered_at: *triggered_at,
                                effect_rate: *effect_rate,
                            },
                        )
                    })
            })
    }
    pub fn iter_group_toggle_states(
        &self,
    ) -> impl Iterator<Item = (ButtonGroupId, GroupToggleState)> + '_ {
        self.iter_groups()
            .map(|(group, toggle_state, _)| (group.id(), toggle_state))
    }
    fn find_active_effects<'a, T, F>(&'a self, extract_effect_fn: F) -> FxIndexMap<&'a T, Rate>
    where
        T: Eq + std::hash::Hash,
        F: Fn(&ButtonAction) -> Option<&T>,
    {
        let mut effects = FxIndexMap::default();

        for (group_info, button_info) in self.iter_info() {
            if let Some(effect) = extract_effect_fn(&button_info.button.on_action) {
                match group_info.group.button_type {
                    ButtonType::Flash => {
                        match button_info.note_state {
                            NoteState::On => effects.insert(effect, button_info.effect_rate),
                            NoteState::Off => effects.shift_remove(&effect),
                        };
                    }
                    ButtonType::Switch => match button_info.note_state {
                        NoteState::On => {
                            effects.shift_remove(&effect);
                            effects.insert(effect, button_info.effect_rate);
                        }
                        NoteState::Off => {}
                    },
                    ButtonType::Toggle => match button_info.note_state {
                        NoteState::On => {
                            if GroupToggleState::On(button_info.button.coordinate)
                                == group_info.toggle_state
                            {
                                effects.insert(effect, button_info.effect_rate);
                            }
                        }
                        NoteState::Off => {}
                    },
                }
            }
        }

        effects
    }
    pub fn pressed_buttons(&self) -> FxHashMap<&ButtonMapping, ButtonStateValue> {
        self.iter_info().fold(
            FxHashMap::default(),
            |mut pressed_buttons, (_, button_info)| {
                match button_info.note_state {
                    NoteState::On => pressed_buttons.insert(
                        button_info.button,
                        (button_info.triggered_at, button_info.effect_rate),
                    ),
                    NoteState::Off => pressed_buttons.remove(button_info.button),
                };
                pressed_buttons
            },
        )
    }
    pub fn pressed_coords(&self) -> FxHashSet<ButtonCoordinate> {
        self.pressed_buttons()
            .into_iter()
            .map(|(button, _)| button.coordinate)
            .collect()
    }
    pub fn global_color(&self) -> Option<Color> {
        let mut on_colors: Vec<(ButtonCoordinate, Color)> = Vec::new();
        let mut last_off: Option<(ButtonCoordinate, Color)> = None;

        let color_buttons =
            self.iter_info().flat_map(|(group_info, button_info)| {
                match button_info.button.on_action {
                    ButtonAction::UpdateGlobalColor(color) => match group_info.group.button_type {
                        ButtonType::Switch => {
                            Some((button_info.button.coordinate, button_info.note_state, color))
                        }
                        _ => panic!("only switch button type implemented for colors"),
                    },
                    _ => None,
                }
            });

        for (coordinate, state, color) in color_buttons {
            match state {
                NoteState::On => {
                    on_colors.push((coordinate, color));
                }
                NoteState::Off => {
                    shift_remove_vec(&mut on_colors, &(coordinate, color));
                    last_off = Some((coordinate, color));
                }
            }
        }

        on_colors
            .last()
            .or_else(|| last_off.as_ref())
            .map(|(_, color)| *color)
    }
    pub fn secondary_color(&self) -> Option<Color> {
        self.iter_info()
            .filter_map(
                |(group_info, button_info)| match button_info.button.on_action {
                    ButtonAction::UpdateGlobalSecondaryColor(color) => {
                        match group_info.group.button_type {
                            ButtonType::Toggle => Some((
                                button_info.button.coordinate,
                                group_info.toggle_state,
                                color,
                            )),
                            _ => panic!("only toggle button type implemented for secondary colors"),
                        }
                    }
                    _ => None,
                },
            )
            .filter_map(|(note, toggle_state, color)| {
                if GroupToggleState::On(note) == toggle_state {
                    Some(color)
                } else {
                    None
                }
            })
            .last()
    }
    pub fn base_position(&self) -> Option<BasePosition> {
        self.find_active_effects(|action| match action {
            ButtonAction::UpdateBasePosition(position) => Some(position),
            _ => None,
        })
        .keys()
        .last()
        .map(|position| **position)
    }
    pub fn active_dimmer_effects(&self) -> FxIndexMap<&DimmerEffect, Rate> {
        self.find_active_effects(|action| match action {
            ButtonAction::ActivateDimmerEffect(effect) => Some(effect),
            _ => None,
        })
    }
    pub fn active_color_effects(&self) -> FxIndexMap<&ColorEffect, Rate> {
        self.find_active_effects(|action| match action {
            ButtonAction::ActivateColorEffect(effect) => Some(effect),
            _ => None,
        })
    }
    pub fn active_pixel_effects(&self) -> FxIndexMap<&PixelEffect, Rate> {
        self.find_active_effects(|action| match action {
            ButtonAction::ActivatePixelEffect(effect) => Some(effect),
            _ => None,
        })
    }
    pub fn active_position_effects(&self) -> FxIndexMap<&PositionEffect, Rate> {
        self.find_active_effects(|action| match action {
            ButtonAction::ActivatePositionEffect(effect) => Some(effect),
            _ => None,
        })
    }
    pub fn iter_states_mut(&mut self) -> impl Iterator<Item = &mut ButtonStateMap> {
        self.group_states.values_mut().map(|(_, states)| states)
    }
    fn button_group_value_mut(&mut self, group: &ButtonGroup) -> &mut GroupStatesValue {
        // If this button group already exists, just use a reference, otherwise clone the button group and insert it

        if self.group_states.contains_key(group) {
            self.group_states.get_mut(group).unwrap()
        } else {
            self.group_states
                .entry(group.clone())
                .or_insert_with(|| (GroupToggleState::Off, FxIndexMap::default()))
        }
    }
    pub fn button_group_state_mut(&mut self, group: &ButtonGroup) -> &mut ButtonStateMap {
        let (_, button_states) = self.button_group_value_mut(group);
        button_states
    }
    pub fn toggle_button_group(&mut self, group: &ButtonGroup, coordinate: ButtonCoordinate) {
        let (toggle_state, _) = self.button_group_value_mut(group);
        toggle_state.toggle_mut(coordinate);
    }
    pub fn update_button_state(
        &mut self,
        group: &ButtonGroup,
        button: ButtonMapping,
        note_state: NoteState,
        now: Instant,
    ) {
        if note_state == NoteState::On {
            self.toggle_button_group(group, button.coordinate);
        }

        let button_states = self.button_group_state_mut(group);
        let key = (button, note_state);

        let previous_state = button_states.shift_remove(&key);
        let effect_rate = previous_state
            .map(|(_, rate)| rate)
            .unwrap_or_else(|| Rate::default());
        button_states.insert(key, (now, effect_rate));
    }
    pub fn update_pressed_button_rates(&mut self, rate: Rate) -> usize {
        let pressed_coords = self.pressed_coords();

        for button_states in self.iter_states_mut() {
            for ((button, _), (_, button_rate)) in button_states.iter_mut() {
                if pressed_coords.contains(&button.coordinate) {
                    *button_rate = rate;
                }
            }
        }

        pressed_coords.len()
    }
}

pub struct ButtonGroupInfo<'a> {
    pub group: &'a ButtonGroup,
    pub toggle_state: GroupToggleState,
}

pub struct ButtonInfo<'a> {
    pub button: &'a ButtonMapping,
    pub note_state: NoteState,
    pub triggered_at: Instant,
    pub effect_rate: Rate,
}
