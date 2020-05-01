use futures::pin_mut;
use futures::stream::{self, StreamExt};
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

use crate::control::button::{pad_states, AkaiPadState};
use crate::lighting_engine::{ControlEvent, EngineState};

#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

async fn run_tick<'a>(
    state: &mut EngineState<'a>,
    fixtures: &mut Vec<fixture::Fixture>,
    dmx_sender: &async_std::sync::Sender<(i32, [u8; 512])>,
    midi_controller: &control::midi::MidiController,
    started_at: &std::time::Instant,
    current_pad_states: &mut rustc_hash::FxHashMap<midi::Note, AkaiPadState>,
) {
    state.update_fixtures(fixtures);
    for (universe, dmx_data) in fixture::fold_fixture_dmx_data(fixtures.iter()).into_iter() {
        dmx_sender.send((universe as i32, dmx_data)).await;
    }

    let new_pad_states = pad_states(
        midi_controller.midi_mapping.pad_mappings().collect(),
        &state
            .control_fixture_group_state()
            .button_states
            .iter_group_toggle_states()
            .collect(),
        state.pad_events(),
        started_at.elapsed(),
    );

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

    *current_pad_states = new_pad_states;
}

#[async_std::main]
async fn main() -> Result<(), async_std::io::Error> {
    let project = project::Project::load("./roller_project.toml").await?;
    let mut fixtures = project.fixtures().await?;

    let midi_controller_name = project.midi_controller.as_ref().unwrap();
    let midi_controller =
        control::midi::MidiController::new_for_device_name(midi_controller_name).unwrap();

    let started_at = Instant::now();
    let mut state = EngineState::new(&midi_controller.midi_mapping);

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
    let mut current_pad_states = pad_states(
        midi_controller.midi_mapping.pad_mappings().collect(),
        &state
            .control_fixture_group_state()
            .button_states
            .iter_group_toggle_states()
            .collect(),
        state.pad_events(),
        started_at.elapsed(),
    );
    midi_controller
        .set_pad_colors(current_pad_states.clone())
        .await;

    let ticks = Some(
        utils::tick_stream(Duration::from_millis(1000 / 40))
            .map(|()| Event::Tick)
            .boxed(),
    );
    let control_events = Some(midi_controller.control_events().map(Event::Control).boxed());
    let clock_events = project
        .midi_clock_events()
        .map(|events| events.map(Event::Clock).boxed());

    let events = stream::select_all(
        vec![ticks, control_events, clock_events]
            .into_iter()
            .flatten(),
    );

    pin_mut!(events);

    loop {
        futures::select! {
            event = events.next() => {
                if let Some(event) = event {
                    match event {
                        Event::Tick => {
                            run_tick(
                                &mut state,
                                &mut fixtures,
                                &dmx_sender,
                                &midi_controller,
                                &started_at,
                                &mut current_pad_states,
                            ).await;
                        }
                        Event::Control(event) => {
                            state.apply_event(event);
                        }
                        Event::Clock(event) => state.clock.apply_event(event),
                    }
                }
            }
        };
    }
}
