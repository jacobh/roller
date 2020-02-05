use async_std::prelude::*;
use futures::pin_mut;
use futures::stream::{self, StreamExt};
use rustc_hash::FxHashMap;
use std::time::{Duration, Instant};

mod clock;
mod color;
mod effect;
mod fixture;
mod lighting_engine;
mod midi_control;
mod project;

use crate::clock::{Beats, Clock};
use crate::lighting_engine::{EngineState, LightingEvent};

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

    let mut state = EngineState {
        clock: Clock::new(128.0),
        master_dimmer: 1.0,
        group_dimmers: FxHashMap::default(),
        effect_intensity: 0.0,
        active_dimmer_effects: vec![
            effect::DimmerEffect::new(effect::triangle_down, Beats::new(4.0), 1.0),
            effect::DimmerEffect::new(effect::triangle_down, Beats::new(2.0), 0.8),
        ],
        active_color_effects: vec![effect::ColorEffect::new(
            effect::hue_shift_30,
            Beats::new(5.0),
        )],
        active_buttons: vec![],
    };

    let mut ola_client = ola_client::OlaClient::connect_localhost().await?;

    let (dmx_sender, mut dmx_receiver) = async_std::sync::channel::<(i32, Vec<u8>)>(10);
    async_std::task::spawn(async move {
        while let Some((universe, dmx_data)) = dmx_receiver.next().await {
            ola_client.send_dmx_data(universe, dmx_data).await.unwrap();
        }
    });

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
    let mut pad_states = state.pad_states(&midi_controller.midi_mapping);

    for i in 0..64 {
        midi_controller
            .set_pad_color(i, midi_control::AkaiPadState::Green)
            .await;
        async_std::task::sleep(Duration::from_millis(10)).await;
    }
    async_std::task::sleep(Duration::from_millis(150)).await;
    midi_controller.reset_pads().await;
    for (note, pad_state) in pad_states.iter() {
        midi_controller.set_pad_color(*note, *pad_state).await;
    }

    let ticks = tick_stream().map(|()| Event::Tick);
    let lighting_events = midi_controller.lighting_events().map(Event::Lighting);
    let events = stream::select(ticks, lighting_events);
    pin_mut!(events);

    while let Some(event) = events.next().await {
        match event {
            Event::Tick => {
                state.update_fixtures(&mut fixtures);
                let dmx_data = fold_fixture_dmx_data(fixtures.iter());
                dmx_sender.send((10, dmx_data)).await;

                let new_pad_states = state.pad_states(&midi_controller.midi_mapping);

                // find the pads that have updated since the last tick
                let pad_changeset = new_pad_states.iter().filter(|(note, state)| {
                    pad_states
                        .get(note)
                        .map(|prev_state| state != &prev_state)
                        .unwrap_or(true)
                });

                for (note, state) in pad_changeset {
                    dbg!("SETTING PAD COLOR", note, state);
                    midi_controller.set_pad_color(*note, *state).await;
                }

                pad_states = new_pad_states;
            }
            Event::Lighting(event) => {
                state.apply_event(event);
            }
        }
    }

    unreachable!()
}
