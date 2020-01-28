use async_std::prelude::*;

mod fixture;
mod midi_control;
mod ola_client;
mod project;

fn pad_512(mut vec: Vec<u8>) -> Vec<u8> {
    while vec.len() < 512 {
        vec.push(0)
    }
    vec
}

fn fold_fixture_dmx_data<'a>(fixtures: impl Iterator<Item = &'a fixture::Fixture>) -> Vec<u8> {
    let mut dmx_data: Vec<u8> = (0..512).map(|_| 0).collect();

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

#[async_std::main]
async fn main() -> Result<(), async_std::io::Error> {
    let project = project::Project::load("./roller_project.toml").await?;
    let mut fixtures = project.fixtures().await?;

    // TODO this doesn't change for now
    let master_dimmer = 1.0;

    let mut ola_client = ola_client::OlaClient::connect_localhost().await?;

    for fixture in fixtures.iter_mut() {
        if fixture.profile.is_positionable() {
            fixture.set_position((0.0, 1.0)).unwrap();
        }
    }

    let midi_controller = midi_control::MidiController::new_for_device_name("APC MINI").unwrap();
    let events = midi_controller.lighting_events();
    async_std::task::spawn(async {
        events
            .for_each(|event| {
                dbg!(&event);
                match event {
                    midi_control::LightingEvent::UpdateMasterDimmer { dimmer: _dimmer } => {
                        // update master dimmer
                    }
                }
            })
            .await;
    });

    loop {
        println!("red");
        for fixture in fixtures.iter_mut() {
            fixture.set_dimmer(master_dimmer);
            fixture.set_color((255, 0, 0)).unwrap();
        }
        flush_fixtures(&mut ola_client, fixtures.iter()).await?;
        async_std::task::sleep(std::time::Duration::from_millis(500)).await;

        println!("green");
        for fixture in fixtures.iter_mut() {
            fixture.set_dimmer(master_dimmer);
            fixture.set_color((0, 255, 0)).unwrap();
        }
        flush_fixtures(&mut ola_client, fixtures.iter()).await?;
        async_std::task::sleep(std::time::Duration::from_millis(500)).await;

        println!("blue");
        for fixture in fixtures.iter_mut() {
            fixture.set_dimmer(master_dimmer);
            fixture.set_color((0, 0, 255)).unwrap();
        }
        flush_fixtures(&mut ola_client, fixtures.iter()).await?;
        async_std::task::sleep(std::time::Duration::from_millis(500)).await;
    }

    Ok(())
}
