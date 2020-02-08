use rustc_hash::{FxHashMap, FxHashSet};
use std::time::Instant;

use crate::utils::FxIndexMap;

use crate::{
    clock::{Beats, Clock},
    color::Color,
    control::{
        button::{ButtonAction, ButtonMapping, ButtonType, ToggleState, ButtonGroupId},
        midi::{AkaiPadState, MidiMapping, NoteState},
    },
    effect::{self, ColorEffect, DimmerEffect},
    fixture::Fixture,
    project::FixtureGroupId,
};

#[derive(Debug, Clone, PartialEq)]
pub enum LightingEvent {
    UpdateMasterDimmer { dimmer: f64 },
    UpdateGroupDimmer { group_id: FixtureGroupId, dimmer: f64 },
    UpdateGlobalEffectIntensity(f64),
    UpdateButton(Instant, NoteState, ButtonMapping),
    TapTempo(Instant),
}

pub struct EngineState {
    pub clock: Clock,
    pub master_dimmer: f64,
    pub group_dimmers: FxHashMap<FixtureGroupId, f64>,
    pub effect_intensity: f64,
    pub active_color_effects: Vec<ColorEffect>,
    pub button_states: FxIndexMap<(ButtonMapping, NoteState), (ToggleState, Instant)>,
}
impl EngineState {
    pub fn apply_event(&mut self, event: LightingEvent) {
        // dbg!(&event);
        match event {
            LightingEvent::UpdateMasterDimmer { dimmer } => {
                self.master_dimmer = dimmer;
            }
            LightingEvent::UpdateGlobalEffectIntensity(intensity) => {
                self.effect_intensity = intensity;
            }
            LightingEvent::UpdateGroupDimmer { group_id, dimmer } => {
                self.group_dimmers.insert(group_id, dimmer);
            }
            LightingEvent::UpdateButton(now, state, mapping) => {
                let key = (mapping, state);
                let prev_toggle_state = self
                    .button_states
                    .shift_remove(&key)
                    .map(|(toggle_state, _)| toggle_state)
                    .unwrap_or(ToggleState::Off);
                self.button_states
                    .insert(key, (prev_toggle_state.toggle(), now));
            }
            LightingEvent::TapTempo(now) => {
                self.clock.tap(now);
                dbg!(self.clock.bpm());
            }
        }
    }
    pub fn global_color(&self) -> Color {
        let mut on_colors: Vec<(u8, Color)> = Vec::new();
        let mut last_off: Option<(u8, Color)> = None;

        let color_buttons =
            self.button_states
                .keys()
                .flat_map(|(mapping, state)| match mapping.on_action {
                    ButtonAction::UpdateGlobalColor { color } => Some((mapping.note, state, color)),
                    _ => None,
                });

        for (note, state, color) in color_buttons {
            match state {
                NoteState::On => {
                    on_colors.push((note, color));
                }
                NoteState::Off => {
                    let color_idx = on_colors
                        .iter()
                        .position(|(color_note, _)| *color_note == note);

                    if let Some(color_idx) = color_idx {
                        on_colors.remove(color_idx);
                    }
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
    fn active_dimmer_effects(&self) -> FxHashSet<&DimmerEffect> {
        let mut active_dimmer_effects = FxHashSet::default();

        // TODO button groups
        for ((mapping, state), (toggle_state, _)) in self.button_states.iter() {
            if let ButtonAction::ActivateDimmerEffect(effect) = &mapping.on_action {
                match mapping.button_type {
                    ButtonType::Flash => {
                        match state {
                            NoteState::On => active_dimmer_effects.insert(effect),
                            NoteState::Off => active_dimmer_effects.remove(&effect),
                        };
                    }
                    ButtonType::Switch => match state {
                        NoteState::On => {
                            active_dimmer_effects.insert(effect);
                        }
                        NoteState::Off => {}
                    },
                    ButtonType::Toggle => match state {
                        NoteState::On => {
                            match toggle_state {
                                ToggleState::On => active_dimmer_effects.insert(effect),
                                ToggleState::Off => active_dimmer_effects.remove(&effect),
                            };
                        }
                        NoteState::Off => {}
                    },
                }
            }
        }

        active_dimmer_effects
    }
    pub fn update_fixtures(&self, fixtures: &mut Vec<Fixture>) {
        let clock_snapshot = self.clock.snapshot();
        let global_color = self.global_color();
        let active_dimmer_effects = self.active_dimmer_effects();

        for (i, fixture) in fixtures.iter_mut().enumerate() {
            let clock_snapshot = clock_snapshot.shift(Beats::new(i as f64));

            let effect_dimmer = effect::intensity(
                active_dimmer_effects.iter().fold(1.0, |dimmer, effect| {
                    dimmer * effect.dimmer(&clock_snapshot)
                }),
                self.effect_intensity,
            );

            let color = effect::color_intensity(
                global_color.to_hsl(),
                self.active_color_effects
                    .iter()
                    .fold(global_color.to_hsl(), |color, effect| {
                        effect.color(color, &clock_snapshot)
                    }),
                self.effect_intensity,
            );

            let group_dimmer = *fixture
                .group_id
                .and_then(|group_id| self.group_dimmers.get(&group_id))
                .unwrap_or(&1.0);

            fixture.set_dimmer(self.master_dimmer * group_dimmer * effect_dimmer);
            fixture.set_color(color).unwrap();
        }
    }
    pub fn pad_states(&self, midi_mapping: &MidiMapping) -> FxHashMap<u8, AkaiPadState> {
        let mut state = midi_mapping.initial_pad_states();

        let mut group_notes: FxHashMap<ButtonGroupId, Vec<u8>> = FxHashMap::default();
        for button in midi_mapping.buttons.values() {
            if let Some(group_id) = button.group_id {
                group_notes.entry(group_id).or_default().push(button.note);
            }
        }

        let mut active_group_buttons: FxHashMap<ButtonGroupId, Vec<u8>> = group_notes
            .keys()
            .map(|group_id| (*group_id, Vec::new()))
            .collect();

        for ((mapping, note_state), (toggle_state, _)) in self.button_states.iter() {
            match mapping.button_type {
                ButtonType::Flash => {
                    // TODO groups
                    state.insert(
                        mapping.note,
                        match note_state {
                            NoteState::On => AkaiPadState::Green,
                            NoteState::Off => AkaiPadState::Yellow,
                        },
                    );
                }
                ButtonType::Toggle => {
                    // TODO groups
                    state.insert(
                        mapping.note,
                        match note_state {
                            NoteState::On => match toggle_state {
                                ToggleState::On => AkaiPadState::Green,
                                ToggleState::Off => AkaiPadState::Red,
                            },
                            NoteState::Off => match toggle_state {
                                ToggleState::On => AkaiPadState::Green,
                                ToggleState::Off => AkaiPadState::Yellow,
                            },
                        },
                    );
                }
                ButtonType::Switch => match note_state {
                    NoteState::On => {
                        state.insert(mapping.note, AkaiPadState::Green);

                        if let Some(group_id) = mapping.group_id {
                            let active_group_buttons =
                                active_group_buttons.get_mut(&group_id).unwrap();
                            active_group_buttons.push(mapping.note);

                            if active_group_buttons.len() == 1 {
                                for note in group_notes[&group_id].iter() {
                                    if *note != mapping.note {
                                        state.insert(*note, AkaiPadState::Red);
                                    }
                                }
                            }
                        }
                    }
                    NoteState::Off => {
                        state.insert(mapping.note, AkaiPadState::Green);
                        if let Some(group_id) = mapping.group_id {
                            let active_group_buttons =
                                active_group_buttons.get_mut(&group_id).unwrap();

                            let button_idx = active_group_buttons
                                .iter()
                                .position(|note| *note == mapping.note);
                            if let Some(button_idx) = button_idx {
                                active_group_buttons.remove(button_idx);
                            }

                            if active_group_buttons.is_empty() {
                                for note in group_notes[&group_id].iter() {
                                    if *note != mapping.note {
                                        state.insert(*note, AkaiPadState::Yellow);
                                    }
                                }
                            } else {
                                state.insert(mapping.note, AkaiPadState::Red);
                            }
                        }
                    }
                },
            }
        }

        state
    }
}
