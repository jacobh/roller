use rustc_hash::FxHashMap;
use std::time::Instant;

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
    pub active_buttons: Vec<(Instant, NoteState, ButtonMapping)>,
}
impl EngineState {
    pub fn apply_event(&mut self, event: LightingEvent) {
        dbg!(&event);
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
                self.active_buttons.push((now, state, mapping));
                // TODO garbage collection
            }
            LightingEvent::TapTempo(now) => {
                self.clock.tap(now);
                dbg!(self.clock.bpm());
            }
        }
    }
    pub fn global_color(&self) -> Color {
        self.active_buttons
            .iter()
            .flat_map(|(_, state, mapping)| match state {
                NoteState::On => match mapping.on_action {
                    ButtonAction::UpdateGlobalColor { color } => Some(color),
                },
                NoteState::Off => None,
            })
            .last()
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

        let mut active_group_buttons: FxHashMap<usize, Vec<u8>> = FxHashMap::default();

        for (_, note_state, mapping) in self.active_buttons.iter() {
            match note_state {
                NoteState::On => {
                    state.insert(mapping.note, AkaiPadState::Green);

                    if let Some(group_id) = mapping.group_id {
                        active_group_buttons
                            .entry(group_id)
                            .or_insert_with(|| Vec::new())
                            .push(mapping.note);

                        let notes_in_group = midi_mapping
                            .buttons
                            .values()
                            .filter(|button| button.group_id == Some(group_id))
                            .map(|button| button.note)
                            .filter(|note| *note != mapping.note);

                        for note in notes_in_group {
                            state.insert(note, AkaiPadState::Red);
                        }
                    }
                }
                NoteState::Off => {
                    state.insert(mapping.note, AkaiPadState::Green);

                    if let Some(group_id) = mapping.group_id {
                        // This was the last activated button, so it takes precedence
                        if active_group_buttons[&group_id].last() == Some(&mapping.note) {
                            let notes_in_group = midi_mapping
                                .buttons
                                .values()
                                .filter(|button| button.group_id == Some(group_id))
                                .map(|button| button.note)
                                .filter(|note| *note != mapping.note);

                            for note in notes_in_group {
                                state.insert(note, AkaiPadState::Yellow);
                            }
                        }
                    }
                }
            }
        }

        state
    }
}
