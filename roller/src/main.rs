use futures::pin_mut;
use futures::stream::{self, StreamExt};
use midi::Note;
use rustc_hash::FxHashMap;
use std::time::Duration;

mod clock;
mod color;
mod control;
mod effect;
mod fixture;
mod lighting_engine;
mod project;
mod utils;

use crate::clock::Clock;
use crate::control::button::{pad_states, PadEvent};
use crate::lighting_engine::{EngineState, LightingEvent, SceneId};
use crate::utils::FxIndexMap;

#[async_std::main]
async fn main() -> Result<(), async_std::io::Error> {
    let project = project::Project::load("./roller_project.toml").await?;
    let mut fixtures = project.fixtures().await?;

    let mut state = EngineState {
        clock: Clock::new(128.0),
        master_dimmer: 1.0,
        group_dimmers: FxHashMap::default(),
        dimmer_effect_intensity: 0.5,
        color_effect_intensity: 1.0,
        global_speed_multiplier: 1.0,
        active_scene_id: SceneId::new(1),
        scene_button_states: FxHashMap::default(),
    };

    let mut ola_client = ola_client::OlaClient::connect_localhost().await?;

    let (dmx_sender, mut dmx_receiver) = async_std::sync::channel::<(i32, [u8; 512])>(10);
    async_std::task::spawn(async move {
        while let Some((universe, dmx_data)) = dmx_receiver.next().await {
            ola_client
                .send_dmx_data(universe, dmx_data.to_vec())
                .await
                .unwrap();
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

    let midi_controller = control::midi::MidiController::new_for_device_name("APC MINI").unwrap();
    let mut current_pad_states = pad_states(
        midi_controller.midi_mapping.pad_mappings().collect(),
        state.button_states().iter().map(PadEvent::from),
    );

    for i in 0..64 {
        midi_controller
            .set_pad_color(Note::new(i), control::button::AkaiPadState::Green)
            .await;
        async_std::task::sleep(Duration::from_millis(10)).await;
    }
    async_std::task::sleep(Duration::from_millis(150)).await;
    midi_controller.reset_pads().await;
    for (note, pad_state) in current_pad_states.iter() {
        midi_controller.set_pad_color(*note, *pad_state).await;
    }

    let ticks = utils::tick_stream(Duration::from_millis(1000 / 40)).map(|()| Event::Tick);
    let lighting_events = midi_controller.lighting_events().map(Event::Lighting);
    let events = stream::select(ticks, lighting_events);
    pin_mut!(events);

    while let Some(event) = events.next().await {
        match event {
            Event::Tick => {
                state.update_fixtures(&mut fixtures);
                let dmx_data = fixture::fold_fixture_dmx_data(fixtures.iter());
                dmx_sender.send((10, dmx_data)).await;

                let new_pad_states = pad_states(
                    midi_controller.midi_mapping.pad_mappings().collect(),
                    state.button_states().iter().map(PadEvent::from),
                );

                // find the pads that have updated since the last tick
                let pad_changeset = new_pad_states.iter().filter(|(note, state)| {
                    current_pad_states
                        .get(note)
                        .map(|prev_state| state != &prev_state)
                        .unwrap_or(true)
                });

                for (note, state) in pad_changeset {
                    // dbg!("SETTING PAD COLOR", note, state);
                    midi_controller.set_pad_color(*note, *state).await;
                }

                current_pad_states = new_pad_states;
            }
            Event::Lighting(event) => {
                state.apply_event(event);
            }
        }
    }

    unreachable!()
}
