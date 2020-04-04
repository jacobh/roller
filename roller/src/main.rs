use futures::pin_mut;
use futures::stream::{self, StreamExt};
use rustc_hash::FxHashMap;
use std::time::{Duration, Instant};

mod clock;
mod color;
mod control;
mod effect;
mod fixture;
mod lighting_engine;
mod position;
mod project;
mod utils;

use crate::clock::{Clock, Rate};
use crate::control::button::pad_states;
use crate::lighting_engine::{ControlEvent, EngineState, SceneId};

#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[async_std::main]
async fn main() -> Result<(), async_std::io::Error> {
    let project = project::Project::load("./roller_project.toml").await?;
    let mut fixtures = project.fixtures().await?;

    let midi_controller =
        control::midi::MidiController::new_for_device_name(&project.midi_controller.unwrap())
            .unwrap();

    let midi_clock_events = project
        .midi_clock
        .and_then(|clock_name| clock::midi_clock_events(&clock_name).ok());

    let started_at = Instant::now();
    let mut state = EngineState {
        midi_mapping: &midi_controller.midi_mapping,
        clock: Clock::new(128.0),
        master_dimmer: 1.0,
        group_dimmers: FxHashMap::default(),
        dimmer_effect_intensity: 0.5,
        color_effect_intensity: 1.0,
        global_clock_rate: Rate::new(1.0),
        active_scene_id: SceneId::new(1),
        active_fixture_group_control: None,
        scene_fixture_group_button_states: FxHashMap::default(),
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

    enum Event {
        Tick,
        Control(ControlEvent),
        Clock(clock::ClockEvent),
    }

    midi_controller.run_pad_startup().await;
    let elapsed = started_at.elapsed();
    let mut current_pad_states = pad_states(
        midi_controller.midi_mapping.pad_mappings().collect(),
        &state
            .control_button_states()
            .iter_group_toggle_states()
            .collect(),
        state.pad_events(),
    )
    .into_iter()
    .map(|(note, pad)| (note, pad.state(elapsed)))
    .collect::<FxHashMap<_, _>>();
    midi_controller
        .set_pad_colors(current_pad_states.clone())
        .await;

    let ticks = Some(
        utils::tick_stream(Duration::from_millis(1000 / 40))
            .map(|()| Event::Tick)
            .boxed(),
    );
    let control_events = Some(midi_controller.control_events().map(Event::Control).boxed());
    let clock_events = midi_clock_events.map(|events| events.map(Event::Clock).boxed());

    let events = stream::select_all(
        vec![ticks, control_events, clock_events]
            .into_iter()
            .flatten(),
    );

    pin_mut!(events);

    while let Some(event) = events.next().await {
        match event {
            Event::Tick => {
                state.update_fixtures(&mut fixtures);
                for (universe, dmx_data) in fixture::fold_fixture_dmx_data(&fixtures).into_iter() {
                    dmx_sender.send((universe as i32, dmx_data)).await;
                }

                let elapsed = started_at.elapsed();
                let new_pad_states = pad_states(
                    midi_controller.midi_mapping.pad_mappings().collect(),
                    &state
                        .control_button_states()
                        .iter_group_toggle_states()
                        .collect(),
                    state.pad_events(),
                )
                .into_iter()
                .map(|(note, pad)| (note, pad.state(elapsed)))
                .collect::<FxHashMap<_, _>>();

                midi_controller
                    .set_pad_colors(
                        // find the pads that have updated since the last tick
                        new_pad_states
                            .iter()
                            .filter(|(note, state)| {
                                current_pad_states
                                    .get(note)
                                    .map(|prev_state| state != &prev_state)
                                    .unwrap_or(true)
                            })
                            .map(|(note, state)| (*note, *state)),
                    )
                    .await;

                current_pad_states = new_pad_states;
            }
            Event::Control(event) => {
                state.apply_event(event);
            }
            Event::Clock(event) => state.clock.apply_event(event),
        }
    }

    unreachable!()
}
