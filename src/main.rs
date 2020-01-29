use async_std::prelude::*;
use std::time::{Instant, Duration};
use futures::pin_mut;
use futures::stream::{self, StreamExt};

mod fixture;
mod midi_control;
mod ola_client;
mod project;

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
        async_std::task::sleep(
            if now < until {
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

    let mut master_dimmer = 1.0;

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
                for fixture in fixtures.iter_mut() {
                    fixture.set_dimmer(master_dimmer);
                    fixture.set_color((255, 0, 0)).unwrap();
                }
                flush_fixtures(&mut ola_client, fixtures.iter()).await
                    .expect("flush_fixtures");
            }
            Event::Lighting(LightingEvent::UpdateMasterDimmer { dimmer }) => {
                dbg!(&dimmer);
                master_dimmer = dimmer;
            }
        }
    }

    unreachable!()
}
