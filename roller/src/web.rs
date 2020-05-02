use async_std::sync::{Mutex, Receiver, Sender};
use broadcaster::BroadcastChannel;
use futures::prelude::*;
use rustc_hash::FxHashMap;
use std::sync::Arc;
use warp::{
    ws::{self, WebSocket, Ws},
    Filter,
};

use midi::{MidiEvent, Note};
use roller_protocol::{
    ButtonCoordinate, ButtonGridLocation, ButtonState, ClientMessage, ServerMessage,
};

use crate::{
    control::{button::AkaiPadState, midi::MidiMapping},
    ControlEvent,
};

// Specific for akai apc mini. really the internal button location should be specific with coords
fn coordinate_to_note(loc: &ButtonGridLocation, coord: &ButtonCoordinate) -> Note {
    match loc {
        ButtonGridLocation::Main => Note::new((8 * coord.row_idx + coord.column_idx) as u8),
        ButtonGridLocation::MetaRight => Note::new((89 - coord.row_idx) as u8),
        ButtonGridLocation::MetaBottom => Note::new((64 + coord.column_idx) as u8),
    }
}

fn note_to_coordinate(note: Note) -> Option<(ButtonGridLocation, ButtonCoordinate)> {
    let note = u8::from(note) as usize;
    if note < 64 {
        Some((
            ButtonGridLocation::Main,
            ButtonCoordinate {
                row_idx: note / 8,
                column_idx: note % 8,
            },
        ))
    } else if note < 72 {
        Some((
            ButtonGridLocation::MetaBottom,
            ButtonCoordinate {
                row_idx: 0,
                column_idx: note - 64,
            },
        ))
    } else if note < 90 {
        Some((
            ButtonGridLocation::MetaRight,
            ButtonCoordinate {
                row_idx: 89 - note,
                column_idx: 0,
            },
        ))
    } else {
        None
    }
}

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
    server_message_channel: BroadcastChannel<ServerMessage>,
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
        server_message_channel.map(Event::ServerMessage),
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
                        let note = coordinate_to_note(&loc, &coord);

                        let midi_event = MidiEvent::NoteOn {
                            note,
                            velocity: 100,
                        };

                        let control_event = midi_mapping.midi_to_control_event(&midi_event);
                        if let Some(control_event) = control_event {
                            event_sender.send(control_event).await;
                        }
                    }
                    ClientMessage::ButtonReleased(loc, coord) => {
                        let note = coordinate_to_note(&loc, &coord);

                        let midi_event = MidiEvent::NoteOff {
                            note,
                            velocity: 100,
                        };

                        let control_event = midi_mapping.midi_to_control_event(&midi_event);
                        if let Some(control_event) = control_event {
                            event_sender.send(control_event).await;
                        }
                    }
                };
            }
        }
    }
}

pub fn serve_frontend(
    midi_mapping: Arc<MidiMapping>,
    initial_pad_states: &FxHashMap<Note, AkaiPadState>,
    mut pad_state_update_recv: Receiver<(Note, AkaiPadState)>,
    event_sender: Sender<ControlEvent>,
) {
    let initial_button_states: Arc<Mutex<FxHashMap<_, _>>> = Arc::new(Mutex::new(
        initial_pad_states
            .iter()
            .filter_map(|(note, pad_state)| {
                let coord = note_to_coordinate(*note);
                if let Some(coord) = coord {
                    Some((coord, akai_pad_state_to_button_state(pad_state)))
                } else {
                    None
                }
            })
            .collect(),
    ));

    let server_message_channel: BroadcastChannel<ServerMessage> = BroadcastChannel::new();

    // Update initial button states with incoming messages
    let initial_button_states2 = initial_button_states.clone();
    let server_message_channel2 = server_message_channel.clone();
    async_std::task::spawn(async move {
        while let Some((note, state)) = pad_state_update_recv.next().await {
            let coord = note_to_coordinate(note);
            let state = akai_pad_state_to_button_state(&state);
            if let Some((loc, coord)) = coord {
                let mut states = initial_button_states2.lock().await;
                states.insert((loc.clone(), coord.clone()), state.clone());

                let message = ServerMessage::ButtonStatesUpdated(vec![(loc, coord, state)]);
                server_message_channel2.send(&message).await.unwrap();
            }
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
            let server_message_channel = server_message_channel.clone();

            ws.on_upgrade(move |websocket| {
                browser_session(
                    websocket,
                    midi_mapping,
                    initial_button_states,
                    server_message_channel,
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
