use futures::pin_mut;
use futures::stream::{self, StreamExt};
use rustc_hash::FxHashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use structopt::StructOpt;

use roller_protocol::{ButtonCoordinate, ButtonGridLocation, ButtonState, InputEvent};

mod clock;
mod color;
mod control;
mod effect;
mod fixture;
mod lighting_engine;
mod position;
mod project;
mod utils;
mod web;

use crate::control::button::{pad_states, AkaiPadState};
use crate::lighting_engine::EngineState;

#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[derive(StructOpt, Debug)]
#[structopt(name = "roller")]
struct CliArgs {
    #[structopt(short, long, default_value = "roller_project.toml", parse(from_os_str))]
    config: PathBuf,
    #[structopt(long, default_value = "localhost:9010")]
    ola_host: String,
}

async fn run_tick<'a>(
    state: &mut EngineState<'a>,
    fixtures: &mut Vec<fixture::Fixture>,
    dmx_sender: &async_std::sync::Sender<(i32, [u8; 512])>,
    midi_controller: Option<&control::midi::MidiController>,
    started_at: &std::time::Instant,
    current_pad_states: &mut rustc_hash::FxHashMap<
        (ButtonGridLocation, ButtonCoordinate),
        AkaiPadState,
    >,
    web_pad_state_update_send: &async_std::sync::Sender<
        Vec<(ButtonGridLocation, ButtonCoordinate, ButtonState)>,
    >,
) {
    state.update_fixtures(fixtures);
    for (universe, dmx_data) in fixture::fold_fixture_dmx_data(fixtures.iter()).into_iter() {
        dmx_sender.send((universe as i32, dmx_data)).await;
    }

    let new_pad_states = pad_states(
        state.control_mapping.pad_mappings().collect(),
        &state
            .control_fixture_group_state()
            .button_states
            .iter_group_toggle_states()
            .collect(),
        state.pad_events(),
        started_at.elapsed(),
    );

    // find the pads that have updated since the last tick
    let changed_pad_states: Vec<(ButtonGridLocation, ButtonCoordinate, AkaiPadState)> =
        new_pad_states
            .iter()
            .filter(|(location_coord, state)| {
                current_pad_states
                    .get(location_coord)
                    .map(|prev_state| state != &prev_state)
                    .unwrap_or(true)
            })
            .map(|((loc, coord), state)| (*loc, *coord, *state))
            .collect();

    // temporary shim
    let changed_button_states = changed_pad_states
        .into_iter()
        .map(|(loc, coord, state)| (loc, coord, akai_pad_state_to_button_state(&state)))
        .collect::<Vec<_>>();

    if let Some(midi_controller) = midi_controller {
        midi_controller
            .set_button_states(changed_button_states.clone().into_iter())
            .await;
    }

    web_pad_state_update_send.send(changed_button_states).await;

    *current_pad_states = new_pad_states;
}

#[async_std::main]
async fn main() -> Result<(), async_std::io::Error> {
    let args = CliArgs::from_args();

    let project = project::Project::load(args.config).await?;
    let mut fixtures = project.fixtures().await?;

    let midi_controller = match project.midi_controller.as_ref() {
        Some(midi_controller_name) => {
            control::midi::MidiController::new_for_device_name(midi_controller_name).ok()
        }
        None => None,
    };

    let started_at = Instant::now();
    let control_mapping = control::default_control_mapping();
    let mut state = EngineState::new(&control_mapping);

    let mut ola_client = ola_client::OlaClient::connect(&args.ola_host)
        .await
        .expect(&format!("Ola server at {} is not running", &args.ola_host));

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
        Input(InputEvent),
        Clock(clock::ClockEvent),
    }

    let mut current_pad_states = pad_states(
        control_mapping.pad_mappings().collect(),
        &state
            .control_fixture_group_state()
            .button_states
            .iter_group_toggle_states()
            .collect(),
        state.pad_events(),
        started_at.elapsed(),
    );

    // temporary shim
    let initial_button_states: FxHashMap<(ButtonGridLocation, ButtonCoordinate), ButtonState> =
        current_pad_states
            .iter()
            .map(|(loc_coord, state)| (*loc_coord, akai_pad_state_to_button_state(state)))
            .collect();

    if let Some(midi_controller) = &midi_controller {
        midi_controller.run_pad_startup().await;
        midi_controller
            .set_button_states(
                initial_button_states
                    .iter()
                    .map(|((loc, coord), val)| (*loc, *coord, *val)),
            )
            .await;
    }

    let (web_input_events_send, web_input_events_recv) = async_std::sync::channel::<InputEvent>(64);
    let (web_pad_state_update_send, web_pad_state_update_recv) =
        async_std::sync::channel::<Vec<(ButtonGridLocation, ButtonCoordinate, ButtonState)>>(64);

    let web_input_events = Some(
        web_input_events_recv
            .map(|event| Event::Input(event))
            .boxed(),
    );

    let ticks = Some(
        utils::tick_stream(Duration::from_millis(1000 / 40))
            .map(|()| Event::Tick)
            .boxed(),
    );
    let input_events = midi_controller
        .as_ref()
        .map(|controller| controller.input_events().map(Event::Input).boxed());
    let clock_events = project
        .midi_clock_events()
        .map(|events| events.map(Event::Clock).boxed());

    let events = stream::select_all(
        vec![ticks, input_events, clock_events, web_input_events]
            .into_iter()
            .flatten(),
    );

    pin_mut!(events);

    web::serve_frontend(
        &initial_button_states,
        web_pad_state_update_recv,
        web_input_events_send,
    );

    while let Some(event) = events.next().await {
        match event {
            Event::Tick => {
                run_tick(
                    &mut state,
                    &mut fixtures,
                    &dmx_sender,
                    midi_controller.as_ref(),
                    &started_at,
                    &mut current_pad_states,
                    &web_pad_state_update_send,
                )
                .await;
            }
            Event::Input(event) => {
                state.apply_input_event(event);
            }
            Event::Clock(event) => state.clock.apply_event(event),
        }
    }
    unreachable!()
}

// temporary shim
fn akai_pad_state_to_button_state(state: &AkaiPadState) -> ButtonState {
    match state {
        AkaiPadState::Off => ButtonState::Unused,
        AkaiPadState::Green => ButtonState::Active,
        AkaiPadState::GreenBlink => ButtonState::Active,
        AkaiPadState::Red => ButtonState::Deactivated,
        AkaiPadState::RedBlink => ButtonState::Deactivated,
        AkaiPadState::Yellow => ButtonState::Inactive,
        AkaiPadState::YellowBlink => ButtonState::Inactive,
    }
}
