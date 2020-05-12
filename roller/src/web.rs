use async_std::sync::{Mutex, Receiver, Sender};
use broadcaster::BroadcastChannel;
use futures::prelude::*;
use rustc_hash::FxHashMap;
use std::sync::Arc;
use warp::{
    ws::{self, WebSocket, Ws},
    Filter,
};

use roller_protocol::{
    ButtonCoordinate, ButtonGridLocation, ButtonState, ClientMessage, ServerMessage,
};

use crate::{
    control::{button::AkaiPadState, midi::MidiMapping},
    ControlEvent,
};

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

async fn browser_session(
    websocket: WebSocket,
    midi_mapping: Arc<MidiMapping>,
    initial_button_states: FxHashMap<(ButtonGridLocation, ButtonCoordinate), ButtonState>,
    server_message_recv: impl Stream<Item = ServerMessage> + Unpin,
    event_sender: Sender<ControlEvent>,
) {
    let (mut tx, rx) = websocket.split();

    // Send through initial button states
    let initial_states_message = ServerMessage::ButtonStatesUpdated(
        initial_button_states
            .into_iter()
            .map(|((loc, coord), state)| (loc, coord, state))
            .collect(),
    );

    let msg = bincode::serialize::<ServerMessage>(&initial_states_message).unwrap();
    tx.send(ws::Message::binary(msg)).await.unwrap();

    enum Event {
        ServerMessage(ServerMessage),
        ClientMessage(Result<ws::Message, warp::Error>),
    }

    let mut events = stream::select(
        rx.map(Event::ClientMessage),
        server_message_recv.map(Event::ServerMessage),
    );

    while let Some(event) = events.next().await {
        match event {
            Event::ServerMessage(msg) => {
                let msg = bincode::serialize::<ServerMessage>(&msg).unwrap();
                match tx.send(ws::Message::binary(msg)).await {
                    Ok(()) => {}
                    Err(_) => {
                        dbg!("Client has hung up");
                        return;
                    }
                }
            }
            Event::ClientMessage(Err(e)) => {
                println!("error reading from client: {:?}", e);
                return;
            }
            Event::ClientMessage(Ok(msg)) => {
                if !msg.is_binary() {
                    continue;
                }

                let msg = bincode::deserialize::<ClientMessage>(msg.as_bytes())
                    .expect("bincode::deserialize");

                println!("{:?}", msg);

                match msg {
                    ClientMessage::ButtonPressed(loc, coord) => {
                        let control_event = midi_mapping.button_press_to_control_event(loc, coord);
                        if let Some(control_event) = control_event {
                            event_sender.send(control_event).await;
                        }
                    }
                    ClientMessage::ButtonReleased(loc, coord) => {
                        let control_event =
                            midi_mapping.button_release_to_control_event(loc, coord);
                        if let Some(control_event) = control_event {
                            event_sender.send(control_event).await;
                        }
                    }
                    ClientMessage::FaderUpdated(id, value) => {
                        if let Some(fader) = midi_mapping.faders.get(&id) {
                            event_sender.send(fader.control_event(value)).await;
                        }
                    }
                };
            }
        }
    }
}

pub fn serve_frontend(
    midi_mapping: Arc<MidiMapping>,
    initial_pad_states: &FxHashMap<(ButtonGridLocation, ButtonCoordinate), AkaiPadState>,
    mut pad_state_update_recv: Receiver<Vec<(ButtonGridLocation, ButtonCoordinate, AkaiPadState)>>,
    event_sender: Sender<ControlEvent>,
) {
    let initial_button_states: Arc<
        Mutex<FxHashMap<(ButtonGridLocation, ButtonCoordinate), ButtonState>>,
    > = Arc::new(Mutex::new(
        initial_pad_states
            .into_iter()
            .map(|(loc_coord, state)| (*loc_coord, akai_pad_state_to_button_state(&state)))
            .collect(),
    ));

    let server_message_channel: BroadcastChannel<ServerMessage> = BroadcastChannel::new();

    // Update initial button states with incoming messages
    let initial_button_states2 = initial_button_states.clone();
    let (mut server_message_sender, _) = server_message_channel.clone().split();
    async_std::task::spawn(async move {
        while let Some(coord_states) = pad_state_update_recv.next().await {
            // remap akai states to button states
            let coord_states: Vec<_> = coord_states
                .into_iter()
                .map(|(loc, coord, state)| (loc, coord, akai_pad_state_to_button_state(&state)))
                .collect();

            let mut states = initial_button_states2.lock().await;
            for (loc, coord, state) in coord_states.iter() {
                states.insert((loc.clone(), coord.clone()), state.clone());
            }

            let message = ServerMessage::ButtonStatesUpdated(coord_states);
            server_message_sender.send(message).await.unwrap();
        }
    });

    let index = warp::get().and(warp::fs::file("web_ui/index.html"));
    let assets = warp::get().and(warp::fs::dir("web_ui"));

    let websocket = warp::get()
        .and(warp::path("ws"))
        .and(warp::ws())
        .map(move |ws: Ws| {
            let midi_mapping = midi_mapping.clone();
            let event_sender = event_sender.clone();
            let initial_button_states =
                async_std::task::block_on(initial_button_states.lock()).clone();
            let (_, server_message_recv) = server_message_channel.clone().split();

            ws.on_upgrade(move |websocket| {
                browser_session(
                    websocket,
                    midi_mapping,
                    initial_button_states,
                    server_message_recv,
                    event_sender,
                )
            })
        });

    let app = warp::path::end().and(index).or(websocket).or(assets);

    let mut rt = tokio::runtime::Runtime::new().unwrap();

    std::thread::spawn(move || {
        rt.block_on(async {
            warp::serve(app).bind(([0, 0, 0, 0], 8888)).await;
        });
    });
}
