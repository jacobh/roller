use derive_more::Constructor;
use midi::Note;
use rustc_hash::{FxHashMap, FxHashSet};
use std::time::Instant;

use crate::{
    clock::{Clock, ClockOffsetOptionExt, Rate},
    color::Color,
    control::{
        button::{
            ButtonAction, ButtonGroup, ButtonGroupId, ButtonMapping, ButtonType, GroupToggleState,
            MetaButtonAction, PadEvent,
        },
        midi::{MidiMapping, NoteState},
    },
    effect::{self, ColorEffect, DimmerEffect, PixelEffect, PixelRangeSet, PositionEffect},
    fixture::Fixture,
    position::BasePosition,
    project::FixtureGroupId,
    utils::{shift_remove_vec, FxIndexMap},
};

type ButtonStateMap = FxIndexMap<(ButtonMapping, NoteState), ButtonStateValue>;
type ButtonStateValue = (Instant, Rate);

type ButtonGroupStateMap = FxHashMap<ButtonGroupId, (ButtonType, GroupToggleState, ButtonStateMap)>;
type FixtureGroupStateMap = FxHashMap<FixtureGroupId, ButtonGroupStateMap>;

#[derive(Default)]
pub struct SceneState {
    // contains base effect states, for all fixtures
    pub base: ButtonGroupStateMap,
    // Contains states for effects enabled for specific groups. These take
    // precedence over any effects set in the `default` state
    pub fixture_groups: FixtureGroupStateMap,
}
impl SceneState {
    fn fixture_group_ids(&self) -> impl Iterator<Item = FixtureGroupId> + '_ {
        self.fixture_groups.keys().copied()
    }
    fn base_button_states(&self) -> &ButtonGroupStateMap {
        &self.base
    }
    fn fixture_group_button_states(
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
}

// This is just for the case where no buttons have been activated yet
lazy_static::lazy_static! {
    static ref EMPTY_BUTTON_GROUP_STATES: ButtonGroupStateMap = FxHashMap::default();
    static ref EMPTY_SCENE_BUTTON_STATES: SceneState = SceneState::default();
    static ref DEFAULT_FIXTURE_GROUP_VALUE: FixtureGroupValue<'static> = FixtureGroupValue::default();
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Constructor)]
pub struct SceneId(usize);

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

struct FixtureGroupValue<'a> {
    global_color: Color,
    secondary_color: Option<Color>,
    active_dimmer_effects: FxIndexMap<&'a DimmerEffect, Rate>,
    active_color_effects: FxIndexMap<&'a ColorEffect, Rate>,
    active_pixel_effects: FxIndexMap<&'a PixelEffect, Rate>,
    active_position_effects: FxIndexMap<&'a PositionEffect, Rate>,
    base_position: BasePosition,
}
impl<'a> Default for FixtureGroupValue<'a> {
    fn default() -> FixtureGroupValue<'a> {
        FixtureGroupValue {
            global_color: Color::Violet,
            secondary_color: None,
            active_dimmer_effects: FxIndexMap::default(),
            active_color_effects: FxIndexMap::default(),
            active_pixel_effects: FxIndexMap::default(),
            active_position_effects: FxIndexMap::default(),
            base_position: BasePosition::default(),
        }
    }
}

fn active_effects<'a, T, F>(
    button_states: impl Iterator<Item = (ButtonGroupInfo, ButtonInfo<'a>)>,
    extract_effect_fn: F,
) -> FxIndexMap<&'a T, Rate>
where
    T: Eq + std::hash::Hash,
    F: Fn(&ButtonAction) -> Option<&T>,
{
    let mut effects = FxIndexMap::default();

    for (group_info, button_info) in button_states {
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
                        if GroupToggleState::On(button_info.button.note) == group_info.toggle_state
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

#[derive(Debug, Clone, PartialEq)]
pub enum LightingEvent {
    UpdateMasterDimmer(f64),
    UpdateGroupDimmer(FixtureGroupId, f64),
    UpdateDimmerEffectIntensity(f64),
    UpdateColorEffectIntensity(f64),
    UpdateClockRate(Rate),
    ActivateScene(SceneId),
    ToggleFixtureGroupControl(FixtureGroupId),
    UpdateButton(ButtonGroup, ButtonMapping, NoteState, Instant),
    TapTempo(Instant),
}

pub struct EngineState<'a> {
    pub midi_mapping: &'a MidiMapping,
    pub clock: Clock,
    pub master_dimmer: f64,
    pub group_dimmers: FxHashMap<FixtureGroupId, f64>,
    pub dimmer_effect_intensity: f64,
    pub color_effect_intensity: f64,
    pub global_clock_rate: Rate,
    pub active_scene_id: SceneId,
    pub active_fixture_group_control: Option<FixtureGroupId>,
    pub scene_fixture_group_button_states: FxHashMap<SceneId, SceneState>,
}
impl<'a> EngineState<'a> {
    fn active_scene_state(&self) -> &SceneState {
        self.scene_fixture_group_button_states
            .get(&self.active_scene_id)
            .unwrap_or_else(|| &*EMPTY_SCENE_BUTTON_STATES)
    }
    fn active_button_states(&self) -> &ButtonGroupStateMap {
        if let Some(group_id) = self.active_fixture_group_control {
            self.active_scene_state()
                .fixture_group_button_states(group_id)
        } else {
            self.active_scene_state().base_button_states()
        }
    }
    pub fn active_button_group_states_mut(&mut self) -> &mut ButtonGroupStateMap {
        let active_scene_state = self
            .scene_fixture_group_button_states
            .entry(self.active_scene_id)
            .or_default();

        if let Some(group_id) = self.active_fixture_group_control {
            active_scene_state
                .fixture_groups
                .entry(group_id)
                .or_default()
        } else {
            &mut active_scene_state.base
        }
    }
    fn pressed_buttons(&self) -> FxHashMap<&ButtonMapping, ButtonStateValue> {
        self.active_scene_state()
            .button_states(self.active_fixture_group_control)
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
    fn pressed_notes(&self) -> FxHashSet<Note> {
        self.pressed_buttons()
            .into_iter()
            .map(|(button, _)| button.note)
            .collect()
    }
    fn update_pressed_button_rates(&mut self, rate: Rate) {
        let pressed_notes = self.pressed_notes();

        let button_states = self
            .active_button_group_states_mut()
            .values_mut()
            .map(|(_, _, states)| states);

        for button_states in button_states {
            for ((button, _), (_, button_rate)) in button_states.iter_mut() {
                if pressed_notes.contains(&button.note) {
                    *button_rate = rate;
                }
            }
        }
    }
    fn button_states_mut(
        &mut self,
        button_group_id: ButtonGroupId,
        button_type: ButtonType,
    ) -> &mut ButtonStateMap {
        let (_, _, button_states) = self
            .active_button_group_states_mut()
            .entry(button_group_id)
            .or_insert_with(|| (button_type, GroupToggleState::Off, FxIndexMap::default()));

        button_states
    }
    pub fn button_group_toggle_states(
        &self,
    ) -> impl Iterator<Item = (ButtonGroupId, GroupToggleState)> + '_ {
        self.active_button_states()
            .iter()
            .map(|(group_id, (_, toggle_state, _))| (*group_id, *toggle_state))
    }
    fn toggle_button_group(&mut self, id: ButtonGroupId, button_type: ButtonType, note: Note) {
        self.active_button_group_states_mut()
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
    fn update_button_state(
        &mut self,
        group: &ButtonGroup,
        button: ButtonMapping,
        note_state: NoteState,
        now: Instant,
    ) {
        if note_state == NoteState::On {
            self.toggle_button_group(group.id, group.button_type, button.note);
        }

        let button_states = self.button_states_mut(group.id, group.button_type);
        let key = (button, note_state);

        let previous_state = button_states.shift_remove(&key);
        let effect_rate = previous_state
            .map(|(_, rate)| rate)
            .unwrap_or_else(|| Rate::default());
        button_states.insert(key, (now, effect_rate));
    }
    pub fn apply_event(&mut self, event: LightingEvent) {
        // dbg!(&event);
        match event {
            LightingEvent::UpdateMasterDimmer(dimmer) => {
                self.master_dimmer = dimmer;
            }
            LightingEvent::UpdateDimmerEffectIntensity(intensity) => {
                self.dimmer_effect_intensity = intensity;
            }
            LightingEvent::UpdateColorEffectIntensity(intensity) => {
                self.color_effect_intensity = intensity;
            }
            LightingEvent::UpdateClockRate(rate) => {
                let pressed_notes = self.pressed_notes();

                // If there are any buttons currently pressed, update the rate of those buttons, note the global rate
                if !pressed_notes.is_empty() {
                    self.update_pressed_button_rates(rate);
                } else {
                    self.global_clock_rate = rate;
                }
            }
            LightingEvent::ActivateScene(scene_id) => {
                self.active_scene_id = scene_id;
            }
            LightingEvent::ToggleFixtureGroupControl(group_id) => {
                if Some(group_id) == self.active_fixture_group_control {
                    self.active_fixture_group_control = None;
                } else {
                    self.active_fixture_group_control = Some(group_id);
                }
            }
            LightingEvent::UpdateGroupDimmer(group_id, dimmer) => {
                self.group_dimmers.insert(group_id, dimmer);
            }
            LightingEvent::UpdateButton(group, mapping, note_state, now) => {
                self.update_button_state(&group, mapping, note_state, now);
            }
            LightingEvent::TapTempo(now) => {
                self.clock.tap(now);
                dbg!(self.clock.bpm());
            }
        }
    }
    pub fn global_color(&self, fixture_group_id: Option<FixtureGroupId>) -> Color {
        let mut on_colors: Vec<(Note, Color)> = Vec::new();
        let mut last_off: Option<(Note, Color)> = None;

        let color_buttons = self
            .active_scene_state()
            .button_states(fixture_group_id)
            .flat_map(
                |(group_info, button_info)| match button_info.button.on_action {
                    ButtonAction::UpdateGlobalColor(color) => match group_info.button_type {
                        ButtonType::Switch => {
                            Some((button_info.button.note, button_info.note_state, color))
                        }
                        _ => panic!("only switch button type implemented for colors"),
                    },
                    _ => None,
                },
            );

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
            .unwrap_or_else(|| Color::Violet)
    }
    pub fn secondary_color(&self, fixture_group_id: Option<FixtureGroupId>) -> Option<Color> {
        self.active_scene_state()
            .button_states(fixture_group_id)
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
    fn base_position(&self, fixture_group_id: Option<FixtureGroupId>) -> BasePosition {
        active_effects(
            self.active_scene_state().button_states(fixture_group_id),
            |action| match action {
                ButtonAction::UpdateBasePosition(position) => Some(position),
                _ => None,
            },
        )
        .keys()
        .last()
        .map(|position| **position)
        .unwrap_or_default()
    }
    fn active_dimmer_effects(
        &self,
        fixture_group_id: Option<FixtureGroupId>,
    ) -> FxIndexMap<&DimmerEffect, Rate> {
        active_effects(
            self.active_scene_state().button_states(fixture_group_id),
            |action| match action {
                ButtonAction::ActivateDimmerEffect(effect) => Some(effect),
                _ => None,
            },
        )
    }
    fn active_color_effects(
        &self,
        fixture_group_id: Option<FixtureGroupId>,
    ) -> FxIndexMap<&ColorEffect, Rate> {
        active_effects(
            self.active_scene_state().button_states(fixture_group_id),
            |action| match action {
                ButtonAction::ActivateColorEffect(effect) => Some(effect),
                _ => None,
            },
        )
    }
    fn active_pixel_effects(
        &self,
        fixture_group_id: Option<FixtureGroupId>,
    ) -> FxIndexMap<&PixelEffect, Rate> {
        active_effects(
            self.active_scene_state().button_states(fixture_group_id),
            |action| match action {
                ButtonAction::ActivatePixelEffect(effect) => Some(effect),
                _ => None,
            },
        )
    }
    fn active_position_effects(
        &self,
        fixture_group_id: Option<FixtureGroupId>,
    ) -> FxIndexMap<&PositionEffect, Rate> {
        active_effects(
            self.active_scene_state().button_states(fixture_group_id),
            |action| match action {
                ButtonAction::ActivatePositionEffect(effect) => Some(effect),
                _ => None,
            },
        )
    }
    fn fixture_group_value(
        &'a self,
        fixture_group_id: Option<FixtureGroupId>,
    ) -> FixtureGroupValue<'a> {
        FixtureGroupValue {
            global_color: self.global_color(fixture_group_id),
            secondary_color: self.secondary_color(fixture_group_id),
            active_dimmer_effects: self.active_dimmer_effects(fixture_group_id),
            active_color_effects: self.active_color_effects(fixture_group_id),
            active_pixel_effects: self.active_pixel_effects(fixture_group_id),
            active_position_effects: self.active_position_effects(fixture_group_id),
            base_position: self.base_position(fixture_group_id),
        }
    }
    fn fixture_group_values(&'a self) -> FxHashMap<FixtureGroupId, FixtureGroupValue<'a>> {
        self.active_scene_state()
            .fixture_group_ids()
            .map(|fixture_group_id| {
                (
                    fixture_group_id,
                    self.fixture_group_value(Some(fixture_group_id)),
                )
            })
            .collect()
    }
    pub fn update_fixtures(&self, fixtures: &mut Vec<Fixture>) {
        let clock_snapshot = self.clock.snapshot().with_rate(self.global_clock_rate);
        let fixture_group_values = self.fixture_group_values();

        let fixture_values = fixtures
            .iter()
            .map(|fixture| {
                // TODO assuming all fixtures are in a group
                let values = fixture_group_values
                    .get(&fixture.group_id.unwrap())
                    .unwrap_or_else(|| &*DEFAULT_FIXTURE_GROUP_VALUE);

                let effect_dimmer = if fixture.dimmer_effects_enabled() {
                    values
                        .active_dimmer_effects
                        .iter()
                        .fold(1.0, |dimmer, (effect, rate)| {
                            dimmer
                                * effect::compress(
                                    effect.dimmer(&effect.clock_offset.offsetted_for_fixture(
                                        &clock_snapshot.with_rate(*rate),
                                        &fixture,
                                        &fixtures,
                                    )),
                                    self.dimmer_effect_intensity,
                                )
                        })
                } else {
                    1.0
                };

                let (base_color, secondary_color) = if fixture.group_id
                    == Some(FixtureGroupId::new(1))
                    || fixture.group_id == Some(FixtureGroupId::new(2))
                {
                    (
                        values.global_color.to_hsl(),
                        values.secondary_color.map(Color::to_hsl),
                    )
                } else {
                    if let Some(secondary_color) = values.secondary_color {
                        (secondary_color.to_hsl(), Some(values.global_color.to_hsl()))
                    } else {
                        (values.global_color.to_hsl(), None)
                    }
                };

                let color = if fixture.color_effects_enabled() {
                    effect::color_intensity(
                        base_color,
                        values.active_color_effects.iter().fold(
                            base_color,
                            |color, (effect, rate)| {
                                effect.color(
                                    color,
                                    secondary_color,
                                    &effect.clock_offset.offsetted_for_fixture(
                                        &clock_snapshot.with_rate(*rate),
                                        &fixture,
                                        &fixtures,
                                    ),
                                )
                            },
                        ),
                        self.color_effect_intensity,
                    )
                } else {
                    base_color
                };

                let pixel_range_set: Option<PixelRangeSet> = if fixture.pixel_effects_enabled() {
                    // TODO only using first active pixel effect
                    values
                        .active_pixel_effects
                        .iter()
                        .nth(0)
                        .map(|(effect, rate)| {
                            effect.pixel_range_set(&effect.clock_offset.offsetted_for_fixture(
                                &clock_snapshot.with_rate(*rate),
                                &fixture,
                                &fixtures,
                            ))
                        })
                } else {
                    None
                };

                let position = if fixture.position_effects_enabled() {
                    Some(
                        values
                            .active_position_effects
                            .iter()
                            .map(|(effect, rate)| {
                                effect.position(&effect.clock_offset.offsetted_for_fixture(
                                    &clock_snapshot.with_rate(*rate),
                                    &fixture,
                                    &fixtures,
                                ))
                            })
                            .fold(
                                values.base_position.for_fixture(&fixture, &fixtures),
                                |position1, position2| position1 + position2,
                            ),
                    )
                } else {
                    None
                };

                let group_dimmer = fixture
                    .group_id
                    .and_then(|group_id| self.group_dimmers.get(&group_id).copied())
                    .unwrap_or(1.0);

                let dimmer = self.master_dimmer * group_dimmer * effect_dimmer;
                (dimmer, color, pixel_range_set, position)
            })
            .collect::<Vec<_>>();

        for (fixture, (dimmer, color, pixel_range, position)) in
            fixtures.iter_mut().zip(fixture_values)
        {
            fixture.set_dimmer(dimmer);
            fixture.set_color(color).unwrap();

            if fixture.profile.beam_count() > 1 {
                if let Some(pixel_range) = pixel_range {
                    fixture
                        .set_beam_dimmers(&pixel_range.pixel_dimmers(fixture.profile.beam_count()))
                } else {
                    // If there's no active pixel effect, reset pixels
                    fixture.set_all_beam_dimmers(1.0);
                }
            }

            if let Some(position) = position {
                fixture.set_position(position).unwrap();
            }
        }
    }
    fn meta_pad_events(&self) -> impl Iterator<Item = PadEvent<'_>> {
        let active_scene_button = self
            .midi_mapping
            .meta_buttons
            .values()
            .find(|button| {
                button.on_action == MetaButtonAction::ActivateScene(self.active_scene_id)
            })
            .unwrap();

        let active_fixture_group_toggle_button =
            self.active_fixture_group_control
                .map(|control_fixture_group_id| {
                    self.midi_mapping
                        .meta_buttons
                        .values()
                        .find(|button| {
                            button.on_action
                                == MetaButtonAction::ToggleFixtureGroupControl(
                                    control_fixture_group_id,
                                )
                        })
                        .unwrap()
                });

        let pressed_button_rate: Option<Rate> =
            self.pressed_buttons().values().map(|(_, rate)| *rate).max();

        let active_clock_rate_button = self
            .midi_mapping
            .meta_buttons
            .values()
            .find(|button| {
                button.on_action
                    == MetaButtonAction::UpdateClockRate(
                        pressed_button_rate.unwrap_or(self.global_clock_rate),
                    )
            })
            .unwrap();

        vec![
            Some(active_scene_button),
            Some(active_clock_rate_button),
            active_fixture_group_toggle_button,
        ]
        .into_iter()
        .flatten()
        .map(PadEvent::new_on)
    }
    pub fn pad_events(&self) -> impl Iterator<Item = PadEvent<'_>> {
        self.active_scene_state()
            .button_states(self.active_fixture_group_control)
            .map(PadEvent::from)
            .chain(self.meta_pad_events())
    }
}
