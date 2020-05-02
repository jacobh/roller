use async_std::sync::Sender;
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
    event_sender: Sender<ControlEvent>,
) {
    let (mut tx, mut rx) = websocket.split();

    // Send through initial button states
    let initial_states_message = ServerMessage::ButtonStatesUpdated(
        initial_button_states
            .into_iter()
            .map(|((loc, coord), state)| (loc, coord, state))
            .collect(),
    );

    let msg = bincode::serialize::<ServerMessage>(&initial_states_message).unwrap();
    tx.send(ws::Message::binary(msg)).await.unwrap();

    while let Some(message) = rx.next().await {
        match message {
            Err(e) => {
                println!("error reading from client: {:?}", e);
                return;
            }
            Ok(msg) => {
                if !msg.is_binary() {
                    continue;
                }

                let msg = bincode::deserialize::<ClientMessage>(msg.as_bytes())
                    .expect("bincode::deserialize");

                println!("{:?}", msg);

                match msg {
                    ClientMessage::ButtonPressed(loc, coord) => {
                        let note = coordinate_to_note(&loc, &coord);

                        let midi_events = [
                            MidiEvent::NoteOn {
                                note,
                                velocity: 100,
                            },
                            MidiEvent::NoteOff {
                                note,
                                velocity: 100,
                            },
                        ];

                        for midi_event in &midi_events {
                            let control_event = midi_mapping.midi_to_control_event(midi_event);
                            if let Some(control_event) = control_event {
                                event_sender.send(control_event).await;
                            }
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
    event_sender: Sender<ControlEvent>,
) {
    let initial_button_states: FxHashMap<_, _> = initial_pad_states
        .iter()
        .filter_map(|(note, pad_state)| {
            let coord = note_to_coordinate(*note);
            if let Some(coord) = coord {
                Some((coord, akai_pad_state_to_button_state(pad_state)))
            } else {
                None
            }
        })
        .collect();

    let index = warp::get().and(warp::fs::file("web_ui/index.html"));
    let assets = warp::get().and(warp::fs::dir("web_ui"));

    let websocket = warp::get()
        .and(warp::path("ws"))
        .and(warp::ws())
        .map(move |ws: Ws| {
            let midi_mapping = midi_mapping.clone();
            let event_sender = event_sender.clone();
            let initial_button_states = initial_button_states.clone();

            ws.on_upgrade(move |websocket| {
                browser_session(websocket, midi_mapping, initial_button_states, event_sender)
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
