use async_std::sync::Sender;
use futures::prelude::*;
use std::sync::Arc;
use warp::{
    ws::{WebSocket, Ws},
    Filter,
};

use midi::{MidiEvent, Note};
use roller_protocol::{ButtonCoordinate, ButtonGridLocation, ClientMessage};

use crate::{control::midi::MidiMapping, ControlEvent};

// Specific for akai apc mini. really the internal button location should be specific with coords
fn coordinate_to_note(loc: &ButtonGridLocation, coord: &ButtonCoordinate) -> Note {
    match loc {
        ButtonGridLocation::Main => Note::new((8 * coord.row_idx + coord.column_idx) as u8),
        ButtonGridLocation::MetaRight => Note::new((89 - coord.row_idx) as u8),
        ButtonGridLocation::MetaBottom => Note::new((64 + coord.column_idx) as u8),
    }
}

async fn browser_session(
    websocket: WebSocket,
    midi_mapping: Arc<MidiMapping>,
    event_sender: Sender<ControlEvent>,
) {
    let (_tx, mut rx) = websocket.split();

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

pub fn serve_frontend(midi_mapping: Arc<MidiMapping>, event_sender: Sender<ControlEvent>) {
    let index = warp::get().and(warp::fs::file("web_ui/index.html"));
    let assets = warp::get().and(warp::fs::dir("web_ui"));

    let websocket = warp::get()
        .and(warp::path("ws"))
        .and(warp::ws())
        .map(move |ws: Ws| {
            let midi_mapping = midi_mapping.clone();
            let event_sender = event_sender.clone();

            ws.on_upgrade(move |websocket| browser_session(websocket, midi_mapping, event_sender))
        });

    let app = warp::path::end().and(index).or(websocket).or(assets);

    let mut rt = tokio::runtime::Runtime::new().unwrap();

    std::thread::spawn(move || {
        rt.block_on(async {
            warp::serve(app).bind(([0, 0, 0, 0], 8888)).await;
        });
    });
}
