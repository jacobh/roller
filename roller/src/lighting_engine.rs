use itertools::Itertools;
use rustc_hash::FxHashMap;
use std::time::Instant;

use crate::utils::FxIndexMap;

use crate::{
    clock::{Beats, Clock},
    color::Color,
    effect::{self, ColorEffect, DimmerEffect},
    fixture::Fixture,
    midi_control::{AkaiPadState, ButtonAction, ButtonMapping, MidiMapping, NoteState},
};

#[derive(Debug, Clone, PartialEq)]
pub enum LightingEvent {
    UpdateMasterDimmer { dimmer: f64 },
    UpdateGroupDimmer { group_id: usize, dimmer: f64 },
    UpdateGlobalEffectIntensity(f64),
    UpdateButton(Instant, NoteState, ButtonMapping),
    TapTempo(Instant),
}

pub struct EngineState {
    pub clock: Clock,
    pub master_dimmer: f64,
    pub group_dimmers: FxHashMap<usize, f64>,
    pub effect_intensity: f64,
    pub active_dimmer_effects: Vec<DimmerEffect>,
    pub active_color_effects: Vec<ColorEffect>,
    pub button_states: FxIndexMap<(ButtonMapping, NoteState), Instant>,
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
                // If this is a note on event, remove any past note off events to avoid
                // confusion of a note off event coming before any note on event
                if state == NoteState::On {
                    self.button_states
                        .shift_remove(&(mapping.clone(), NoteState::Off));
                }

                let key = (mapping, state);
                self.button_states.shift_remove(&key);
                self.button_states.insert(key, now);
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
                        .position(|(color_note, _)| *color_note == note)
                        .unwrap();

                    on_colors.remove(color_idx);
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
    pub fn update_fixtures(&self, fixtures: &mut Vec<Fixture>) {
        let clock_snapshot = self.clock.snapshot();
        let global_color = self.global_color();

        for (i, fixture) in fixtures.iter_mut().enumerate() {
            let clock_snapshot = clock_snapshot.shift(Beats::new(i as f64));

            let effect_dimmer = effect::intensity(
                self.active_dimmer_effects
                    .iter()
                    .fold(1.0, |dimmer, effect| {
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

        let group_notes: FxHashMap<usize, Vec<u8>> = midi_mapping
            .buttons
            .values()
            .group_by(|button| button.group_id)
            .into_iter()
            .flat_map(|(group_id, buttons)| {
                group_id.map(|group_id| (group_id, buttons.map(|button| button.note).collect()))
            })
            .collect();

        let mut active_group_buttons: FxHashMap<usize, Vec<u8>> = FxHashMap::default();

        for (mapping, note_state) in self.button_states.keys() {
            match note_state {
                NoteState::On => {
                    state.insert(mapping.note, AkaiPadState::Green);

                    if let Some(group_id) = mapping.group_id {
                        active_group_buttons
                            .entry(group_id)
                            .or_insert_with(|| Vec::new())
                            .push(mapping.note);

                        if active_group_buttons[&group_id].len() == 1 {
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
                        // remove button
                        let button_idx = active_group_buttons[&group_id]
                            .iter()
                            .position(|note| *note == mapping.note)
                            .unwrap();
                        active_group_buttons
                            .get_mut(&group_id)
                            .unwrap()
                            .remove(button_idx);

                        if active_group_buttons[&group_id].is_empty() {
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
            }
        }

        state
    }
}