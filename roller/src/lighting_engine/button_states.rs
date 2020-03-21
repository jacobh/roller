use midi::Note;
use rustc_hash::{FxHashMap, FxHashSet};
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
    pub static ref EMPTY_BUTTON_GROUP_STATES: ButtonGroupStateMap = FxHashMap::default();
    pub static ref EMPTY_SCENE_STATE: SceneState = SceneState::default();
}

pub type ButtonStateMap = FxIndexMap<(ButtonMapping, NoteState), ButtonStateValue>;
pub type ButtonStateValue = (Instant, Rate);
pub type ButtonGroupStateMap =
    FxHashMap<ButtonGroupId, (ButtonType, GroupToggleState, ButtonStateMap)>;
pub type FixtureGroupStateMap = FxHashMap<FixtureGroupId, ButtonGroupStateMap>;

#[derive(Default)]
pub struct SceneState {
    // contains base effect states, for all fixtures
    pub base: ButtonGroupStateMap,
    // Contains states for effects enabled for specific groups. These take
    // precedence over any effects set in the `default` state
    pub fixture_groups: FixtureGroupStateMap,
}
impl SceneState {
    pub fn fixture_group_ids(&self) -> impl Iterator<Item = FixtureGroupId> + '_ {
        self.fixture_groups.keys().copied()
    }
    pub fn base_button_states(&self) -> &ButtonGroupStateMap {
        &self.base
    }
    pub fn fixture_group_button_states(
        &self,
        fixture_group_id: FixtureGroupId,
    ) -> &ButtonGroupStateMap {
        self.fixture_groups
            .get(&fixture_group_id)
            .unwrap_or_else(|| &*EMPTY_BUTTON_GROUP_STATES)
    }
    pub fn button_states(
        &self,
        fixture_group_id: Option<FixtureGroupId>,
    ) -> impl Iterator<Item = (ButtonGroupInfo, ButtonInfo<'_>)> {
        fixture_group_id
            .map(|group_id| self.fixture_group_button_states(group_id))
            .unwrap_or_else(|| self.base_button_states())
            .iter()
            .flat_map(|(group_id, (button_type, toggle_state, button_states))| {
                button_group_states_info(*group_id, *button_type, *toggle_state, button_states)
            })
    }
    pub fn button_group_states_mut(
        &mut self,
        fixture_group_id: Option<FixtureGroupId>,
    ) -> &mut ButtonGroupStateMap {
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
        self.button_states(active_fixture_group_control).fold(
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

// Takes a button group and returns an iterator of `Info` summaries
fn button_group_states_info(
    group_id: ButtonGroupId,
    button_type: ButtonType,
    toggle_state: GroupToggleState,
    button_states: &ButtonStateMap,
) -> impl Iterator<Item = (ButtonGroupInfo, ButtonInfo<'_>)> {
    button_states
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
