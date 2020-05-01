use futures::prelude::*;
use warp::{
    ws::{WebSocket, Ws},
    Filter,
};

use midi::Note;
use roller_protocol::{ButtonCoordinate, ButtonGridLocation, ClientMessage};

use crate::ControlEvent;

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
    event_sender: async_std::sync::Sender<ControlEvent>,
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

                let control_event = match msg {
                    ClientMessage::ButtonPressed(loc, coord) => {
                        let _note = coordinate_to_note(&loc, &coord);
                        // TODO we need the button mapping here to determine which button was pressed
                        // Placeholder
                        ControlEvent::TapTempo(std::time::Instant::now())
                    }
                };
                event_sender.send(control_event).await;
            }
        }
    }
}

pub fn serve_frontend(event_sender: async_std::sync::Sender<ControlEvent>) {
    let index = warp::get().and(warp::fs::file("web_ui/index.html"));
    let assets = warp::get().and(warp::fs::dir("web_ui"));

    let websocket = warp::get()
        .and(warp::path("ws"))
        .and(warp::ws())
        .and(warp::any().map(move || event_sender.clone()))
        .map(
            move |ws: Ws, sender: async_std::sync::Sender<ControlEvent>| {
                ws.on_upgrade(move |websocket| browser_session(websocket, sender))
            },
        );

    let app = warp::path::end().and(index).or(websocket).or(assets);

    let mut rt = tokio::runtime::Runtime::new().unwrap();

    std::thread::spawn(move || {
        rt.block_on(async {
            warp::serve(app).bind(([0, 0, 0, 0], 8888)).await;
        });
    });
}
