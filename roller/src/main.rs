use async_std::prelude::*;
use futures::pin_mut;
use futures::stream::{self, StreamExt};
use rustc_hash::FxHashMap;
use std::time::{Duration, Instant};

mod clock;
mod color;
mod effect;
mod fixture;
mod midi_control;
mod project;

use crate::clock::{Beats, Clock};
use crate::midi_control::LightingEvent;

fn fold_fixture_dmx_data<'a>(fixtures: impl Iterator<Item = &'a fixture::Fixture>) -> Vec<u8> {
    let mut dmx_data: Vec<u8> = vec![0; 512];

    for fixture in fixtures {
        for (channel, value) in fixture.absolute_dmx().into_iter().enumerate() {
            if let Some(value) = value {
                dmx_data[channel] = value
            }
        }
    }

    dmx_data
}

async fn flush_fixtures<'a>(
    client: &mut ola_client::OlaClient,
    fixtures: impl Iterator<Item = &'a fixture::Fixture>,
) -> Result<(), async_std::io::Error> {
    let dmx_data = fold_fixture_dmx_data(fixtures);
    client.send_dmx_data(10, dmx_data).await
}

fn tick_stream() -> impl Stream<Item = ()> {
    let mut next_tick_at = Instant::now();

    stream::repeat(()).then(move |()| {
        let until = next_tick_at;
        next_tick_at += Duration::from_millis(1000 / 40);
        let now = Instant::now();
        async_std::task::sleep(if now < until {
            until - now
        } else {
            Duration::from_secs(0)
        })
    })
}

struct EngineState {
    clock: Clock,
    master_dimmer: f64,
    group_dimmers: FxHashMap<usize, f64>,
    global_color: color::Color,
    effect_intensity: f64,
    active_dimmer_effects: Vec<effect::DimmerEffect>,
    active_color_effects: Vec<effect::ColorEffect>,
}
impl EngineState {
    fn apply_event(&mut self, event: LightingEvent) {
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
    fn update_fixtures(&self, fixtures: &mut Vec<fixture::Fixture>) {
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

#[async_std::main]
async fn main() -> Result<(), async_std::io::Error> {
    let project = project::Project::load("./roller_project.toml").await?;
    let mut fixtures = project.fixtures().await?;

    let mut state = EngineState {
        clock: Clock::new(128.0),
        master_dimmer: 1.0,
        group_dimmers: FxHashMap::default(),
        global_color: color::Color::Violet,
        effect_intensity: 0.0,
        active_dimmer_effects: vec![
            effect::DimmerEffect::new(effect::triangle_down, Beats::new(4.0), 1.0),
            effect::DimmerEffect::new(effect::triangle_down, Beats::new(2.0), 0.8),
        ],
        active_color_effects: vec![effect::ColorEffect::new(
            effect::hue_shift_30,
            Beats::new(5.0),
        )],
    };

    let mut ola_client = ola_client::OlaClient::connect_localhost().await?;

    for fixture in fixtures.iter_mut() {
        if fixture.profile.is_positionable() {
            fixture.set_position((0.0, 1.0)).unwrap();
        }
    }

    enum Event {
        Tick,
        Lighting(LightingEvent),
    }

    let midi_controller = midi_control::MidiController::new_for_device_name("APC MINI").unwrap();

    let ticks = tick_stream().map(|()| Event::Tick);
    let lighting_events = midi_controller.lighting_events().map(Event::Lighting);
    let events = stream::select(ticks, lighting_events);
    pin_mut!(events);

    while let Some(event) = events.next().await {
        match event {
            Event::Tick => {
                state.update_fixtures(&mut fixtures);
                flush_fixtures(&mut ola_client, fixtures.iter())
                    .await
                    .expect("flush_fixtures");
            }
            Event::Lighting(event) => {
                state.apply_event(event);
            }
        }
    }

    unreachable!()
}
