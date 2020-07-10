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
    control::{ButtonCoordinate, ButtonGridLocation, ButtonState, InputEvent},
    fixture::{FixtureId, FixtureParams, FixtureState},
    ClientMessage, ServerMessage,
};

async fn browser_session(
    websocket: WebSocket,
    fixture_params: FxHashMap<FixtureId, FixtureParams>,
    initial_button_states: FxHashMap<(ButtonGridLocation, ButtonCoordinate), (String, ButtonState)>,
    server_message_recv: impl Stream<Item = ServerMessage> + Unpin,
    event_sender: Sender<InputEvent>,
) {
    let (mut tx, rx) = websocket.split();

    // Send through initial button states and labels
    let initial_messages = &[
        ServerMessage::ButtonStatesUpdated(
            initial_button_states
                .iter()
                .map(|((loc, coord), (_, state))| (*loc, *coord, *state))
                .collect(),
        ),
        ServerMessage::ButtonLabelsUpdated(
            initial_button_states
                .iter()
                .map(|((loc, coord), (label, _))| (*loc, *coord, label.clone()))
                .collect(),
        ),
        ServerMessage::FixtureParamsUpdated(fixture_params.into_iter().collect()),
    ];

    for message in initial_messages {
        let msg = bincode::serialize::<ServerMessage>(&message).unwrap();
        tx.send(ws::Message::binary(msg)).await.unwrap();
    }

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
                    ClientMessage::Input(input_event) => {
                        event_sender.send(input_event).await;
                    }
                };
            }
        }
    }
}

pub fn serve_frontend(
    initial_button_states: FxHashMap<(ButtonGridLocation, ButtonCoordinate), (String, ButtonState)>,
    fixture_params: FxHashMap<FixtureId, FixtureParams>,
    mut pad_state_update_recv: Receiver<Vec<(ButtonGridLocation, ButtonCoordinate, ButtonState)>>,
    mut fixture_state_updates_recv: Receiver<Vec<(FixtureId, FixtureState)>>,
    event_sender: Sender<InputEvent>,
) {
    let initial_button_states = Arc::new(Mutex::new(initial_button_states));
    let server_message_channel: BroadcastChannel<ServerMessage> = BroadcastChannel::new();

    // Update initial button states with incoming messages
    let initial_button_states2 = initial_button_states.clone();
    let (mut server_message_sender, _) = server_message_channel.clone().split();
    async_std::task::spawn(async move {
        while let Some(coord_states) = pad_state_update_recv.next().await {
            let mut states = initial_button_states2.lock().await;
            for (loc, coord, state) in coord_states.iter() {
                states
                    .entry((loc.clone(), coord.clone()))
                    .and_modify(|(_label, prev_state)| *prev_state = *state)
                    .or_insert_with(|| (String::new(), *state));
            }

            let message = ServerMessage::ButtonStatesUpdated(coord_states);
            server_message_sender.send(message).await.unwrap();
        }
    });

    // broadcast fixture states
    let (mut server_message_sender, _) = server_message_channel.clone().split();
    async_std::task::spawn(async move {
        while let Some(updates) = fixture_state_updates_recv.next().await {
            server_message_sender
                .send(ServerMessage::FixtureStatesUpdated(updates))
                .await
                .unwrap();
        }
    });

    let index = warp::get().and(warp::fs::file("web_ui/index.html"));
    let assets = warp::get().and(warp::fs::dir("web_ui"));

    let websocket = warp::get()
        .and(warp::path("ws"))
        .and(warp::ws())
        .map(move |ws: Ws| {
            let fixture_params = fixture_params.clone();
            let event_sender = event_sender.clone();
            let initial_button_states =
                async_std::task::block_on(initial_button_states.lock()).clone();
            let (_, server_message_recv) = server_message_channel.clone().split();

            ws.on_upgrade(move |websocket| {
                browser_session(
                    websocket,
                    fixture_params,
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
