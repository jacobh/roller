use derive_more::Constructor;
use midi::Note;
use rustc_hash::{FxHashMap, FxHashSet};
use std::time::Instant;

use crate::{
    clock::{Clock, Rate},
    color::Color,
    control::{
        button::{
            ButtonAction, ButtonGroup, ButtonGroupId, ButtonMapping, ButtonType, GroupToggleState,
            MetaButtonAction, PadEvent,
        },
        midi::{MidiMapping, NoteState},
    },
    effect::{self, ColorEffect, DimmerEffect},
    fixture::Fixture,
    project::FixtureGroupId,
    utils::{shift_remove_vec, FxIndexMap},
};

type ButtonStateMap = FxIndexMap<(ButtonMapping, NoteState), ButtonStateValue>;
type ButtonStateValue = (Instant, Rate);

// This is just for the case where no buttons have been activated yet
lazy_static::lazy_static! {
    static ref EMPTY_BUTTON_STATES: ButtonStateMap = {
        FxIndexMap::default()
    };
    static ref EMPTY_GROUP_BUTTON_STATES: FxHashMap<
        ButtonGroupId,
        (ButtonType, GroupToggleState, ButtonStateMap),
    > = FxHashMap::default();
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Constructor)]
pub struct SceneId(usize);

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

#[derive(Debug, Clone, PartialEq)]
pub enum LightingEvent {
    UpdateMasterDimmer(f64),
    UpdateGroupDimmer(FixtureGroupId, f64),
    UpdateDimmerEffectIntensity(f64),
    UpdateColorEffectIntensity(f64),
    UpdateClockRate(Rate),
    ActivateScene(SceneId),
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
    pub scene_group_button_states: FxHashMap<
        SceneId,
        FxHashMap<ButtonGroupId, (ButtonType, GroupToggleState, ButtonStateMap)>,
    >,
}
impl<'a> EngineState<'a> {
    pub fn button_states(&self) -> impl Iterator<Item = (ButtonGroupInfo, ButtonInfo<'_>)> {
        self.scene_group_button_states
            .get(&self.active_scene_id)
            .unwrap_or_else(|| &*EMPTY_GROUP_BUTTON_STATES)
            .iter()
            .flat_map(|(group_id, (button_type, toggle_state, button_states))| {
                button_states.iter().map(
                    move |((button, note_state), (triggered_at, effect_rate))| {
                        (
                            ButtonGroupInfo {
                                id: *group_id,
                                button_type: *button_type,
                                toggle_state: *toggle_state,
                            },
                            ButtonInfo {
                                button: button,
                                note_state: *note_state,
                                triggered_at: *triggered_at,
                                effect_rate: *effect_rate,
                            },
                        )
                    },
                )
            })
    }
    fn pressed_buttons(&self) -> FxHashMap<&ButtonMapping, ButtonStateValue> {
        self.button_states().fold(
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

        let scene_button_states = self
            .scene_group_button_states
            .entry(self.active_scene_id)
            .or_default()
            .values_mut()
            .map(|(_, _, states)| states);

        for button_states in scene_button_states {
            for ((button, _), (_, button_rate)) in button_states.iter_mut() {
                if pressed_notes.contains(&button.note) {
                    *button_rate = rate;
                }
            }
        }
    }
    fn button_states_mut(
        &mut self,
        group_id: ButtonGroupId,
        button_type: ButtonType,
    ) -> &mut ButtonStateMap {
        let (_, _, button_states) = self
            .scene_group_button_states
            .entry(self.active_scene_id)
            .or_default()
            .entry(group_id)
            .or_insert_with(|| (button_type, GroupToggleState::Off, FxIndexMap::default()));

        button_states
    }
    pub fn button_group_toggle_states(
        &self,
    ) -> impl Iterator<Item = (ButtonGroupId, GroupToggleState)> + '_ {
        self.scene_group_button_states
            .get(&self.active_scene_id)
            .unwrap_or_else(|| &*EMPTY_GROUP_BUTTON_STATES)
            .iter()
            .map(|(group_id, (_, toggle_state, _))| (*group_id, *toggle_state))
    }
    fn toggle_button_group(&mut self, id: ButtonGroupId, button_type: ButtonType, note: Note) {
        let button_group_states = self
            .scene_group_button_states
            .entry(self.active_scene_id)
            .or_default();

        button_group_states
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
    pub fn global_color(&self) -> Color {
        let mut on_colors: Vec<(Note, Color)> = Vec::new();
        let mut last_off: Option<(Note, Color)> = None;

        let color_buttons = self.button_states().flat_map(|(group_info, button_info)| {
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
            .unwrap_or_else(|| Color::Violet)
    }
    pub fn secondary_color(&self) -> Option<Color> {
        self.button_states()
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
    fn active_dimmer_effects(&self) -> FxHashMap<&DimmerEffect, Rate> {
        let mut effects = FxHashMap::default();

        // TODO button groups
        for (group_info, button_info) in self.button_states() {
            if let ButtonAction::ActivateDimmerEffect(effect) = &button_info.button.on_action {
                match group_info.button_type {
                    ButtonType::Flash => {
                        match button_info.note_state {
                            NoteState::On => effects.insert(effect, button_info.effect_rate),
                            NoteState::Off => effects.remove(&effect),
                        };
                    }
                    ButtonType::Switch => match button_info.note_state {
                        NoteState::On => {
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
    fn active_color_effects(&self) -> FxHashMap<&ColorEffect, Rate> {
        let mut effects = FxHashMap::default();

        // TODO button groups
        for (group_info, button_info) in self.button_states() {
            if let ButtonAction::ActivateColorEffect(effect) = &button_info.button.on_action {
                match group_info.button_type {
                    ButtonType::Flash => {
                        match button_info.note_state {
                            NoteState::On => effects.insert(effect, button_info.effect_rate),
                            NoteState::Off => effects.remove(&effect),
                        };
                    }
                    ButtonType::Switch => match button_info.note_state {
                        NoteState::On => {
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
    pub fn update_fixtures(&self, fixtures: &mut Vec<Fixture>) {
        let clock_snapshot = self.clock.snapshot().with_rate(self.global_clock_rate);
        let global_color = self.global_color();
        let secondary_color = self.secondary_color().unwrap_or(global_color);
        let active_dimmer_effects = self.active_dimmer_effects();
        let active_color_effects = self.active_color_effects();

        let fixture_values = fixtures
            .iter()
            .map(|fixture| {
                let effect_dimmer =
                    active_dimmer_effects
                        .iter()
                        .fold(1.0, |dimmer, (effect, rate)| {
                            dimmer
                                * effect::compress(
                                    effect.offset_dimmer(
                                        &clock_snapshot.with_rate(*rate),
                                        &fixture,
                                        &fixtures,
                                    ),
                                    self.dimmer_effect_intensity,
                                )
                        });

                let base_color = if fixture.group_id == Some(FixtureGroupId::new(1)) {
                    global_color
                } else {
                    secondary_color
                };

                let color = effect::color_intensity(
                    base_color.to_hsl(),
                    active_color_effects.iter().fold(
                        base_color.to_hsl(),
                        |color, (effect, rate)| {
                            effect.offset_color(
                                color,
                                &clock_snapshot.with_rate(*rate),
                                &fixture,
                                &fixtures,
                            )
                        },
                    ),
                    self.color_effect_intensity,
                );

                let group_dimmer = fixture
                    .group_id
                    .and_then(|group_id| self.group_dimmers.get(&group_id).copied())
                    .unwrap_or(1.0);

                let dimmer = self.master_dimmer * group_dimmer * effect_dimmer;
                (dimmer, color)
            })
            .collect::<Vec<_>>();

        for (fixture, (dimmer, color)) in fixtures.iter_mut().zip(fixture_values) {
            fixture.set_dimmer(dimmer);
            fixture.set_color(color).unwrap();
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
            PadEvent::new_on(active_scene_button),
            PadEvent::new_on(active_clock_rate_button),
        ]
        .into_iter()
    }
    pub fn pad_events(&self) -> impl Iterator<Item = PadEvent<'_>> {
        self.button_states()
            .map(PadEvent::from)
            .chain(self.meta_pad_events())
    }
}
