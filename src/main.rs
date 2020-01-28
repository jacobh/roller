use async_std::prelude::*;
use futures::future::FutureExt;
use std::sync::Mutex;

mod fixture;
mod midi_control;
mod ola_client;
mod project;

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

async fn tick(
    ola_client: &mut ola_client::OlaClient,
    fixtures: &mut Vec<fixture::Fixture>,
    master_dimmer: f64,
) -> Result<(), async_std::io::Error> {
    for fixture in fixtures.iter_mut() {
        fixture.set_dimmer(master_dimmer);
        fixture.set_color((255, 0, 0)).unwrap();
    }
    flush_fixtures(ola_client, fixtures.iter()).await?;
    async_std::task::sleep(std::time::Duration::from_millis(1000 / 40)).await;
    Ok(())
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

    let midi_controller = midi_control::MidiController::new_for_device_name("APC MINI").unwrap();
    let mut events = midi_controller.lighting_events();

    loop {
        futures::future::select(
            tick(&mut ola_client, &mut fixtures, master_dimmer).boxed(),
            // receive incoming midi
            events.next().map(|event| match event {
                Some(midi_control::LightingEvent::UpdateMasterDimmer { dimmer }) => {
                    dbg!(&dimmer);
                    master_dimmer = dimmer
                }
                None => panic!(),
            }),
        )
        .await;
    }
}
