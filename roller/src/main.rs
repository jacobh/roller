use clap::Clap;
use futures::pin_mut;
use futures::stream::{self, StreamExt};
use rustc_hash::FxHashMap;
use std::path::PathBuf;
use std::time::Duration;

use roller_protocol::{
    control::{ButtonState, InputEvent},
    fixture::{fold_fixture_dmx_data, Fixture, FixtureGroupId, FixtureParams, FixtureState},
    lighting_engine::{
        render::{render_fixture_states, FixtureStateRenderContext},
        FixtureGroupState,
    },
    ServerMessage,
};

mod clock;
mod control;
mod fixture;
mod lighting_engine;
mod project;
mod utils;

use crate::control::button::{pad_states, ButtonRef};
use crate::lighting_engine::EngineState;

#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[derive(Clap, Debug)]
#[clap(name = "roller")]
struct CliArgs {
    #[clap(short, long, default_value = "roller_project.toml", parse(from_os_str))]
    config: PathBuf,
    #[clap(long, default_value = "localhost:9010")]
    ola_host: String,
}

async fn run_tick<'a>(
    state: &mut EngineState<'a>,
    fixtures: &mut Vec<Fixture>,
    dmx_sender: &async_std::sync::Sender<(i32, [u8; 512])>,
    midi_controller: Option<&control::midi::MidiController>,
    current_fixture_group_states: &mut (
        FixtureGroupState,
        FxHashMap<FixtureGroupId, FixtureGroupState>,
    ),
    current_button_states: &mut rustc_hash::FxHashMap<ButtonRef<'a>, ButtonState>,
    web_server_message_send: &async_std::sync::Sender<ServerMessage>,
) {
    let (base_state, fixture_group_states) = state.active_scene_state().fixture_group_values();

    let new_fixture_states: Vec<_> = render_fixture_states(
        FixtureStateRenderContext {
            base_state: &base_state,
            fixture_group_states: &fixture_group_states.iter().collect::<Vec<_>>(),
            clock_snapshot: state.clock.snapshot(),
            master_dimmer: state.master_dimmer,
        },
        &fixtures
            .iter()
            .map(|fixture| &fixture.params)
            .collect::<Vec<_>>(),
    )
    .into_iter()
    .map(|(params, state)| (params.id.clone(), state))
    .collect();

    // TODO this is a shim until fixtures is split apart
    for (id, state) in new_fixture_states.clone().into_iter() {
        for fixture in fixtures.iter_mut() {
            if fixture.id() == &id {
                fixture.state = state;
                break;
            }
        }
    }

    for (universe, dmx_data) in fold_fixture_dmx_data(fixtures.iter()).into_iter() {
        dmx_sender.send((universe as i32, dmx_data)).await;
    }

    web_server_message_send
        .send(ServerMessage::FixtureStatesUpdated(new_fixture_states))
        .await;

    // find any fixture group states that have updated since last tick
    let updated_fixture_group_states = {
        let mut states = vec![];

        if current_fixture_group_states.0 != base_state {
            states.push((None, base_state.clone()));
        };

        for (id, new_state) in fixture_group_states.iter() {
            if Some(new_state) != current_fixture_group_states.1.get(id) {
                states.push((Some(*id), new_state.clone()));
            }
        }

        states
    };

    if updated_fixture_group_states.len() > 0 {
        web_server_message_send
            .send(ServerMessage::FixtureGroupStatesUpdated(
                updated_fixture_group_states,
            ))
            .await;
    }

    let new_button_states = pad_states(
        &state.control_mapping,
        &state
            .control_fixture_group_state()
            .button_states
            .iter_group_toggle_states()
            .collect(),
        state.input_events(),
    );

    // find the buttons that have updated since the last tick
    let changed_button_states: Vec<_> = new_button_states
        .iter()
        .filter(|(button_ref, state)| {
            current_button_states
                .get(button_ref)
                .map(|prev_state| state != &prev_state)
                .unwrap_or(true)
        })
        .map(|(button_ref, state)| (button_ref.location(), *button_ref.coordinate(), *state))
        .collect();

    if let Some(midi_controller) = midi_controller {
        midi_controller
            .set_button_states(changed_button_states.clone().into_iter())
            .await;
    }

    if changed_button_states.len() > 0 {
        web_server_message_send
            .send(ServerMessage::ButtonStatesUpdated(changed_button_states))
            .await;
    }

    *current_button_states = new_button_states;
    *current_fixture_group_states = (base_state, fixture_group_states);
}

#[async_std::main]
async fn main() -> Result<(), async_std::io::Error> {
    let args = CliArgs::parse();

    let project = project::Project::load(args.config).await?;
    let mut fixtures = project.fixtures().await?;

    let midi_controller = match project.midi_controller.as_ref() {
        Some(midi_controller_name) => {
            control::midi::MidiController::new_for_device_name(midi_controller_name).ok()
        }
        None => None,
    };

    let control_mapping = control::default_control_mapping();
    let mut state = EngineState::new(&control_mapping);

    let mut ola_client: Option<ola_client::OlaClient> =
        ola_client::OlaClient::connect(&args.ola_host).await.ok();

    let (dmx_sender, mut dmx_receiver) = async_std::sync::channel::<(i32, [u8; 512])>(10);
    async_std::task::spawn(async move {
        while let Some((universe, dmx_data)) = dmx_receiver.next().await {
            // If the ola server is running we will have a client here, otherwise we'll just ignore the incoming data
            if let Some(ola_client) = ola_client.as_mut() {
                ola_client
                    .send_dmx_data(universe, dmx_data.to_vec())
                    .await
                    .unwrap();
            }
        }
    });

    enum Event {
        Tick,
        Input(InputEvent),
        Clock(roller_protocol::clock::ClockEvent),
    }

    let mut current_fixture_group_states = (FixtureGroupState::default(), FxHashMap::default());
    let mut current_button_states = pad_states(
        &control_mapping,
        &state
            .control_fixture_group_state()
            .button_states
            .iter_group_toggle_states()
            .collect(),
        state.input_events(),
    );

    if let Some(midi_controller) = &midi_controller {
        midi_controller.run_pad_startup().await;
        midi_controller
            .set_button_states(
                current_button_states.iter().map(|(button_ref, val)| {
                    (button_ref.location(), *button_ref.coordinate(), *val)
                }),
            )
            .await;
    }

    let (web_input_events_send, web_input_events_recv) = async_std::sync::channel::<InputEvent>(64);
    let (web_server_message_send, web_server_message_recv) =
        async_std::sync::channel::<ServerMessage>(64);

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

    roller_web::serve_frontend(
        current_button_states
            .iter()
            .map(|(button_ref, value)| {
                (
                    (button_ref.location(), *button_ref.coordinate()),
                    (button_ref.label().to_owned(), *value),
                )
            })
            .collect(),
        fixtures
            .clone()
            .into_iter()
            .map(|fixture| (*fixture.id(), fixture.params))
            .collect(),
        web_server_message_recv,
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
                    &mut current_fixture_group_states,
                    &mut current_button_states,
                    &web_server_message_send,
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
