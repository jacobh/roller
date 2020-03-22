use midi::Note;
use rustc_hash::{FxHashMap, FxHashSet};
use std::time::Instant;

use crate::{
    clock::Rate,
    color::Color,
    control::{
        button::{
            ButtonAction, ButtonGroup, ButtonGroupId, ButtonMapping, ButtonType, GroupToggleState,
        },
        midi::NoteState,
    },
    effect::{ColorEffect, DimmerEffect, PixelEffect, PositionEffect},
    lighting_engine::FixtureGroupValue,
    position::BasePosition,
    project::FixtureGroupId,
    utils::{shift_remove_vec, FxIndexMap},
};

// This is just for the case where no buttons have been activated yet
lazy_static::lazy_static! {
    pub static ref EMPTY_BUTTON_GROUP_STATES: ButtonGroupStates = ButtonGroupStates::default();
    pub static ref EMPTY_SCENE_STATE: SceneState = SceneState::default();
}

pub type ButtonStateMap = FxIndexMap<(ButtonMapping, NoteState), ButtonStateValue>;
pub type ButtonStateValue = (Instant, Rate);
pub type FixtureGroupStateMap = FxHashMap<FixtureGroupId, ButtonGroupStates>;

#[derive(Default)]
pub struct SceneState {
    // contains base effect states, for all fixtures
    pub base: ButtonGroupStates,
    // Contains states for effects enabled for specific groups. These take
    // precedence over any effects set in the `default` state
    pub fixture_groups: FixtureGroupStateMap,
}
impl SceneState {
    pub fn base_button_states(&self) -> &ButtonGroupStates {
        &self.base
    }
    pub fn fixture_group_button_states(
        &self,
        fixture_group_id: FixtureGroupId,
    ) -> &ButtonGroupStates {
        self.fixture_groups
            .get(&fixture_group_id)
            .unwrap_or_else(|| &*EMPTY_BUTTON_GROUP_STATES)
    }
    pub fn button_group_states(
        &self,
        fixture_group_id: Option<FixtureGroupId>,
    ) -> &ButtonGroupStates {
        if let Some(group_id) = fixture_group_id {
            self.fixture_group_button_states(group_id)
        } else {
            self.base_button_states()
        }
    }
    pub fn fixture_group_values(
        &self,
    ) -> (
        FixtureGroupValue<'_>,
        FxHashMap<FixtureGroupId, FixtureGroupValue<'_>>,
    ) {
        let base_values = self.base.fixture_group_value();

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
    pub fn button_group_states_mut(
        &mut self,
        fixture_group_id: Option<FixtureGroupId>,
    ) -> &mut ButtonGroupStates {
        if let Some(group_id) = fixture_group_id {
            self.fixture_groups.entry(group_id).or_default()
        } else {
            &mut self.base
        }
    }
}

type GroupStatesValue = (ButtonType, GroupToggleState, ButtonStateMap);
#[derive(Default)]
pub struct ButtonGroupStates {
    group_states: FxHashMap<ButtonGroupId, GroupStatesValue>,
}
impl ButtonGroupStates {
    fn iter(
        &self,
    ) -> impl Iterator<Item = (ButtonGroupId, ButtonType, GroupToggleState, &ButtonStateMap)> {
        self.group_states
            .iter()
            .map(|(group_id, (button_type, toggle_state, states))| {
                (*group_id, *button_type, *toggle_state, states)
            })
    }
    // Takes a button group and returns an iterator of `Info` summaries
    pub fn iter_info(&self) -> impl Iterator<Item = (ButtonGroupInfo, ButtonInfo<'_>)> {
        self.iter()
            .flat_map(|(group_id, button_type, toggle_state, states)| {
                states
                    .iter()
                    .map(move |((button, note_state), (triggered_at, effect_rate))| {
                        (
                            ButtonGroupInfo {
                                id: group_id,
                                button_type: button_type,
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
    pub fn iter_toggle_states(
        &self,
    ) -> impl Iterator<Item = (ButtonGroupId, GroupToggleState)> + '_ {
        self.iter()
            .map(|(group_id, _, toggle_state, _)| (group_id, toggle_state))
    }
    fn find_active_effects<'a, T, F>(&'a self, extract_effect_fn: F) -> FxIndexMap<&'a T, Rate>
    where
        T: Eq + std::hash::Hash,
        F: Fn(&ButtonAction) -> Option<&T>,
    {
        let mut effects = FxIndexMap::default();

        for (group_info, button_info) in self.iter_info() {
            if let Some(effect) = extract_effect_fn(&button_info.button.on_action) {
                match group_info.button_type {
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
                            if GroupToggleState::On(button_info.button.note)
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
    pub fn pressed_notes(&self) -> FxHashSet<Note> {
        self.pressed_buttons()
            .into_iter()
            .map(|(button, _)| button.note)
            .collect()
    }
    pub fn global_color(&self) -> Option<Color> {
        let mut on_colors: Vec<(Note, Color)> = Vec::new();
        let mut last_off: Option<(Note, Color)> = None;

        let color_buttons =
            self.iter_info().flat_map(|(group_info, button_info)| {
                match button_info.button.on_action {
                    ButtonAction::UpdateGlobalColor(color) => match group_info.button_type {
                        ButtonType::Switch => {
                            Some((button_info.button.note, button_info.note_state, color))
                        }
                        _ => panic!("only switch button type implemented for colors"),
                    },
                    _ => None,
                }
            });

        for (note, state, color) in color_buttons {
            match state {
                NoteState::On => {
                    on_colors.push((note, color));
                }
                NoteState::Off => {
                    shift_remove_vec(&mut on_colors, &(note, color));
                    last_off = Some((note, color));
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
                    ButtonAction::UpdateGlobalSecondaryColor(color) => match group_info.button_type
                    {
                        ButtonType::Toggle => {
                            Some((button_info.button.note, group_info.toggle_state, color))
                        }
                        _ => panic!("only toggle button type implemented for secondary colors"),
                    },
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
    pub fn fixture_group_value(&self) -> FixtureGroupValue<'_> {
        FixtureGroupValue {
            global_color: self.global_color(),
            secondary_color: self.secondary_color(),
            active_dimmer_effects: self.active_dimmer_effects(),
            active_color_effects: self.active_color_effects(),
            active_pixel_effects: self.active_pixel_effects(),
            active_position_effects: self.active_position_effects(),
            base_position: self.base_position(),
        }
    }
    pub fn iter_states_mut(&mut self) -> impl Iterator<Item = &mut ButtonStateMap> {
        self.group_states.values_mut().map(|(_, _, states)| states)
    }
    pub fn button_group_state_mut(
        &mut self,
        group_id: ButtonGroupId,
        button_type: ButtonType,
    ) -> &mut ButtonStateMap {
        let (_, _, button_states) = self
            .group_states
            .entry(group_id)
            .or_insert_with(|| (button_type, GroupToggleState::Off, FxIndexMap::default()));

        button_states
    }
    pub fn toggle_button_group(&mut self, id: ButtonGroupId, button_type: ButtonType, note: Note) {
        self.group_states
            .entry(id)
            .and_modify(|(_, toggle_state, _)| {
                toggle_state.toggle_mut(note);
            })
            .or_insert((
                button_type,
                GroupToggleState::On(note),
                FxIndexMap::default(),
            ));
    }
    pub fn update_button_state(
        &mut self,
        group: &ButtonGroup,
        button: ButtonMapping,
        note_state: NoteState,
        now: Instant,
    ) {
        if note_state == NoteState::On {
            self.toggle_button_group(group.id, group.button_type, button.note);
        }

        let button_states = self.button_group_state_mut(group.id, group.button_type);
        let key = (button, note_state);

        let previous_state = button_states.shift_remove(&key);
        let effect_rate = previous_state
            .map(|(_, rate)| rate)
            .unwrap_or_else(|| Rate::default());
        button_states.insert(key, (now, effect_rate));
    }
    pub fn update_pressed_button_rates(&mut self, rate: Rate) -> usize {
        let pressed_notes = self.pressed_notes();

        for button_states in self.iter_states_mut() {
            for ((button, _), (_, button_rate)) in button_states.iter_mut() {
                if pressed_notes.contains(&button.note) {
                    *button_rate = rate;
                }
            }
        }

        pressed_notes.len()
    }
}

pub struct ButtonGroupInfo {
    pub id: ButtonGroupId,
    pub button_type: ButtonType,
    pub toggle_state: GroupToggleState,
}

pub struct ButtonInfo<'a> {
    pub button: &'a ButtonMapping,
    pub note_state: NoteState,
    pub triggered_at: Instant,
    pub effect_rate: Rate,
}
