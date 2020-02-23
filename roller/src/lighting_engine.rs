use derive_more::Constructor;
use midi::Note;
use rustc_hash::{FxHashMap, FxHashSet};
use std::time::Instant;

use crate::{
    clock::{Clock, Rate},
    color::Color,
    control::{
        button::{
            ButtonAction, ButtonMapping, ButtonType, MetaButtonAction, PadEvent, ToggleState,
        },
        midi::{MidiMapping, NoteState},
    },
    effect::{self, ColorEffect, DimmerEffect},
    fixture::Fixture,
    project::FixtureGroupId,
    utils::{FxIndexMap, shift_remove_vec},
};

type ButtonStateMap = FxIndexMap<(ButtonMapping, NoteState), ButtonStateValue>;
type ButtonStateValue = (ToggleState, Instant, Rate);

// This is just for the case where no buttons have been activated yet
lazy_static::lazy_static! {
    static ref EMPTY_BUTTON_STATES: ButtonStateMap = {
        FxIndexMap::default()
    };
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Constructor)]
pub struct SceneId(usize);

#[derive(Debug, Clone, PartialEq)]
pub enum LightingEvent {
    UpdateMasterDimmer(f64),
    UpdateGroupDimmer(FixtureGroupId, f64),
    UpdateDimmerEffectIntensity(f64),
    UpdateColorEffectIntensity(f64),
    UpdateClockRate(Rate),
    ActivateScene(SceneId),
    UpdateButton(Instant, NoteState, ButtonMapping),
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
    pub scene_button_states: FxHashMap<SceneId, ButtonStateMap>,
}
impl<'a> EngineState<'a> {
    pub fn button_states(&self) -> &ButtonStateMap {
        self.scene_button_states
            .get(&self.active_scene_id)
            .unwrap_or_else(|| &*EMPTY_BUTTON_STATES)
    }
    fn pressed_buttons(&self) -> FxHashMap<&ButtonMapping, &ButtonStateValue> {
        self.button_states().iter().fold(
            FxHashMap::default(),
            |mut pressed_buttons, ((button, note_state), value)| {
                match note_state {
                    NoteState::On => pressed_buttons.insert(button, value),
                    NoteState::Off => pressed_buttons.remove(button),
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
    pub fn button_states_mut(&mut self) -> &mut ButtonStateMap {
        self.scene_button_states
            .entry(self.active_scene_id)
            .or_default()
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
                    for ((button, _), (_, _, button_rate)) in self.button_states_mut().iter_mut() {
                        if pressed_notes.contains(&button.note) {
                            *button_rate = rate;
                        }
                    }
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
            LightingEvent::UpdateButton(now, state, mapping) => {
                let key = (mapping, state);
                let prev_toggle_state = self
                    .button_states_mut()
                    .shift_remove(&key)
                    .map(|(toggle_state, _, _)| toggle_state)
                    .unwrap_or(ToggleState::Off);
                self.button_states_mut()
                    .insert(key, (prev_toggle_state.toggle(), now, Rate::default()));
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

        let color_buttons = self
            .button_states()
            .keys()
            .flat_map(|(mapping, state)| match mapping.on_action {
                ButtonAction::UpdateGlobalColor(color) => match mapping.button_type {
                    ButtonType::Switch => Some((mapping.note, state, color)),
                    _ => panic!("only switch button type implemented for colors"),
                },
                _ => None,
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
            .iter()
            .filter_map(
                |((mapping, state), (toggle_state, _, _))| match mapping.on_action {
                    ButtonAction::UpdateGlobalColor(color) => match mapping.button_type {
                        ButtonType::Toggle => Some((mapping.note, state, toggle_state, color)),
                        _ => panic!("only toggle button type implemented for secondary colors"),
                    },
                    _ => None,
                },
            )
            .fold(
                Vec::new(),
                |mut active_colors, (note, note_state, toggle_state, color)| {
                    match note_state {
                        NoteState::On => match toggle_state {
                            ToggleState::On => {
                                active_colors.push((note, color));
                            }
                            ToggleState::Off => {
                                shift_remove_vec(&mut active_colors, &(note, color));
                            }
                        },
                        NoteState::Off => {}
                    };
                    active_colors
                },
            )
            .into_iter()
            .last()
            .map(|(_, color)| color)
    }
    fn active_dimmer_effects(&self) -> FxHashMap<&DimmerEffect, Rate> {
        let mut effects = FxHashMap::default();

        // TODO button groups
        for ((mapping, state), (toggle_state, _, rate)) in self.button_states().iter() {
            if let ButtonAction::ActivateDimmerEffect(effect) = &mapping.on_action {
                match mapping.button_type {
                    ButtonType::Flash => {
                        match state {
                            NoteState::On => effects.insert(effect, *rate),
                            NoteState::Off => effects.remove(&effect),
                        };
                    }
                    ButtonType::Switch => match state {
                        NoteState::On => {
                            effects.insert(effect, *rate);
                        }
                        NoteState::Off => {}
                    },
                    ButtonType::Toggle => match state {
                        NoteState::On => {
                            match toggle_state {
                                ToggleState::On => effects.insert(effect, *rate),
                                ToggleState::Off => effects.remove(&effect),
                            };
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
        for ((mapping, state), (toggle_state, _, rate)) in self.button_states().iter() {
            if let ButtonAction::ActivateColorEffect(effect) = &mapping.on_action {
                match mapping.button_type {
                    ButtonType::Flash => {
                        match state {
                            NoteState::On => effects.insert(effect, *rate),
                            NoteState::Off => effects.remove(&effect),
                        };
                    }
                    ButtonType::Switch => match state {
                        NoteState::On => {
                            effects.insert(effect, *rate);
                        }
                        NoteState::Off => {}
                    },
                    ButtonType::Toggle => match state {
                        NoteState::On => {
                            match toggle_state {
                                ToggleState::On => effects.insert(effect, *rate),
                                ToggleState::Off => effects.remove(&effect),
                            };
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
        let _secondary_color = self.secondary_color();
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

                let color = effect::color_intensity(
                    global_color.to_hsl(),
                    active_color_effects.iter().fold(
                        global_color.to_hsl(),
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

        let pressed_button_rate: Option<Rate> = self
            .pressed_buttons()
            .values()
            .map(|(_, _, rate)| *rate)
            .max();

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
            .iter()
            .map(PadEvent::from)
            .chain(self.meta_pad_events())
    }
}
