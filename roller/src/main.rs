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

use crate::clock::Clock;
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

#[async_std::main]
async fn main() -> Result<(), async_std::io::Error> {
    let project = project::Project::load("./roller_project.toml").await?;
    let mut fixtures = project.fixtures().await?;

    let mut clock = Clock::new(128.0);
    let mut master_dimmer = 1.0;
    let mut group_dimmers: FxHashMap<usize, f64> = FxHashMap::default();
    let mut global_color = color::Color::Violet;
    let mut effect_intensity = 0.75;
    let active_dimmer_effects = vec![effect::DimmerEffect::new(effect::saw_up, 4.0, 0.75)];

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
                let clock_snapshot = clock.snapshot();
                let effect_dimmer = effect::intensity(
                    active_dimmer_effects.iter().fold(1.0, |dimmer, effect| {
                        dimmer * effect.dimmer(&clock_snapshot)
                    }),
                    effect_intensity,
                );

                for fixture in fixtures.iter_mut() {
                    let group_dimmer = *fixture
                        .group_id
                        .and_then(|group_id| group_dimmers.get(&group_id))
                        .unwrap_or(&1.0);

                    fixture.set_dimmer(master_dimmer * group_dimmer * effect_dimmer);
                    fixture.set_color(global_color).unwrap();
                }
                flush_fixtures(&mut ola_client, fixtures.iter())
                    .await
                    .expect("flush_fixtures");
            }
            Event::Lighting(LightingEvent::UpdateMasterDimmer { dimmer }) => {
                dbg!(&dimmer);
                master_dimmer = dimmer;
            }
            Event::Lighting(LightingEvent::UpdateGlobalEffectIntensity(intensity)) => {
                effect_intensity = intensity;
            }
            Event::Lighting(LightingEvent::UpdateGroupDimmer { group_id, dimmer }) => {
                dbg!(&dimmer);
                group_dimmers.insert(group_id, dimmer);
            }
            Event::Lighting(LightingEvent::UpdateGlobalColor { color }) => {
                global_color = dbg!(color);
            }
            Event::Lighting(LightingEvent::TapTempo(now)) => {
                clock.tap(now);
                dbg!(clock.bpm());
            }
        }
    }

    unreachable!()
}
