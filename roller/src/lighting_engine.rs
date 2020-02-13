use midi::Note;
use rustc_hash::{FxHashMap, FxHashSet};
use std::time::Instant;

use crate::{
    clock::Clock,
    color::Color,
    control::{
        button::{ButtonAction, ButtonMapping, ButtonType, ToggleState},
        midi::NoteState,
    },
    effect::{self, ColorEffect, DimmerModifier},
    fixture::Fixture,
    project::FixtureGroupId,
    utils::FxIndexMap,
};

#[derive(Debug, Clone, PartialEq)]
pub enum LightingEvent {
    UpdateMasterDimmer {
        dimmer: f64,
    },
    UpdateGroupDimmer {
        group_id: FixtureGroupId,
        dimmer: f64,
    },
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
        let mut on_colors: Vec<(Note, Color)> = Vec::new();
        let mut last_off: Option<(Note, Color)> = None;

        let color_buttons =
            self.button_states
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
    fn active_dimmer_modifiers(&self) -> FxHashSet<&DimmerModifier> {
        let mut modifiers = FxHashSet::default();

        // TODO button groups
        for ((mapping, state), (toggle_state, _)) in self.button_states.iter() {
            if let ButtonAction::ActivateDimmerModifier(modifier) = &mapping.on_action {
                match mapping.button_type {
                    ButtonType::Flash => {
                        match state {
                            NoteState::On => modifiers.insert(modifier),
                            NoteState::Off => modifiers.remove(&modifier),
                        };
                    }
                    ButtonType::Switch => match state {
                        NoteState::On => {
                            modifiers.insert(modifier);
                        }
                        NoteState::Off => {}
                    },
                    ButtonType::Toggle => match state {
                        NoteState::On => {
                            match toggle_state {
                                ToggleState::On => modifiers.insert(modifier),
                                ToggleState::Off => modifiers.remove(&modifier),
                            };
                        }
                        NoteState::Off => {}
                    },
                }
            }
        }

        modifiers
    }
    pub fn update_fixtures(&self, fixtures: &mut Vec<Fixture>) {
        let clock_snapshot = self.clock.snapshot();
        let global_color = self.global_color();
        let active_dimmer_modifiers = self.active_dimmer_modifiers();

        let fixture_values = fixtures
            .iter()
            .map(|fixture| {
                let effect_dimmer = effect::intensity(
                    active_dimmer_modifiers
                        .iter()
                        .fold(1.0, |dimmer, modifier| {
                            dimmer * modifier.offset_dimmer(&clock_snapshot, &fixture, &fixtures)
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

                let dimmer = self.master_dimmer * group_dimmer * effect_dimmer;
                (dimmer, color)
            })
            .collect::<Vec<_>>();

        for (fixture, (dimmer, color)) in fixtures.iter_mut().zip(fixture_values) {
            fixture.set_dimmer(dimmer);
            fixture.set_color(color).unwrap();
        }
    }
}
