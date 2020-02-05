use rustc_hash::FxHashMap;
use std::time::Instant;

use crate::{
    clock::{Beats, Clock},
    color::Color,
    effect::{self, ColorEffect, DimmerEffect},
    fixture::Fixture,
};

#[derive(Debug, Clone, PartialEq)]
pub enum LightingEvent {
    UpdateMasterDimmer { dimmer: f64 },
    UpdateGroupDimmer { group_id: usize, dimmer: f64 },
    UpdateGlobalColor { color: Color },
    UpdateGlobalEffectIntensity(f64),
    TapTempo(Instant),
}

pub struct EngineState {
    pub clock: Clock,
    pub master_dimmer: f64,
    pub group_dimmers: FxHashMap<usize, f64>,
    pub global_color: Color,
    pub effect_intensity: f64,
    pub active_dimmer_effects: Vec<DimmerEffect>,
    pub active_color_effects: Vec<ColorEffect>,
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
            LightingEvent::UpdateGlobalColor { color } => {
                self.global_color = color;
            }
            LightingEvent::TapTempo(now) => {
                self.clock.tap(now);
                dbg!(self.clock.bpm());
            }
        }
    }
    pub fn update_fixtures(&self, fixtures: &mut Vec<Fixture>) {
        let clock_snapshot = self.clock.snapshot();

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
                self.global_color.to_hsl(),
                self.active_color_effects
                    .iter()
                    .fold(self.global_color.to_hsl(), |color, effect| {
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
}
