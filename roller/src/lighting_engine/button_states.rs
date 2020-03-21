use midi::Note;
use rustc_hash::{FxHashMap, FxHashSet};
use std::collections::hash_map::Entry;
use std::time::Instant;

use crate::{
    clock::Rate,
    control::{
        button::{ButtonGroupId, ButtonMapping, ButtonType, GroupToggleState},
        midi::NoteState,
    },
    project::FixtureGroupId,
    utils::FxIndexMap,
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
    pub fn fixture_group_ids(&self) -> impl Iterator<Item = FixtureGroupId> + '_ {
        self.fixture_groups.keys().copied()
    }
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
    pub fn iter_group_button_info(
        &self,
        fixture_group_id: Option<FixtureGroupId>,
    ) -> impl Iterator<Item = (ButtonGroupInfo, ButtonInfo<'_>)> {
        fixture_group_id
            .map(|group_id| self.fixture_group_button_states(group_id))
            .unwrap_or_else(|| self.base_button_states())
            .iter_info()
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
    pub fn pressed_buttons(
        &self,
        active_fixture_group_control: Option<FixtureGroupId>,
    ) -> FxHashMap<&ButtonMapping, ButtonStateValue> {
        self.iter_group_button_info(active_fixture_group_control)
            .fold(
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
    pub fn pressed_notes(
        &self,
        active_fixture_group_control: Option<FixtureGroupId>,
    ) -> FxHashSet<Note> {
        self.pressed_buttons(active_fixture_group_control)
            .into_iter()
            .map(|(button, _)| button.note)
            .collect()
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
    fn iter_info(&self) -> impl Iterator<Item = (ButtonGroupInfo, ButtonInfo<'_>)> {
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
    pub fn entry(&mut self, group_id: ButtonGroupId) -> Entry<'_, ButtonGroupId, GroupStatesValue> {
        self.group_states.entry(group_id)
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
