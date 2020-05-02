use im_rc::{vector, HashMap, Vector};
use yew::{
    format::Binary,
    prelude::*,
    services::websocket::{WebSocketService, WebSocketStatus, WebSocketTask},
};

use crate::{button_grid::ButtonGrid, console_log, utils::callback_fn};
use roller_protocol::{
    ButtonCoordinate, ButtonGridLocation, ButtonState, ClientMessage, ServerMessage,
};

pub struct App {
    link: ComponentLink<Self>,
    websocket: WebSocketTask,
    button_states: HashMap<ButtonGridLocation, Vector<Vector<ButtonState>>>,
}

#[derive(Debug)]
pub enum AppMsg {
    ButtonPressed(ButtonGridLocation, ButtonCoordinate),
    ServerMessage(ServerMessage),
    NoOp,
}

impl Component for App {
    type Message = AppMsg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut websocket = WebSocketService::new();

        let websocket = websocket
            .connect_binary(
                "ws://localhost:8888/ws",
                link.callback(|msg: Binary| match msg {
                    Ok(buff) => {
                        let msg = bincode::deserialize::<ServerMessage>(&buff)
                            .expect("bincode::deserialize");

                        AppMsg::ServerMessage(msg)
                    }
                    Err(e) => {
                        crate::console_log!("websocket recv error: {:?}", e);
                        AppMsg::NoOp
                    }
                }),
                link.callback(|status: WebSocketStatus| {
                    crate::console_log!("websocket status: {:?}", status);
                    AppMsg::NoOp
                }),
            )
            .expect("websocket.connect_binary");

        let mut button_states = HashMap::new();

        button_states.insert(
            ButtonGridLocation::Main,
            (0..8)
                .map(|_column_idx| (0..8).map(|_row_idx| ButtonState::Unused).collect())
                .collect(),
        );

        button_states.insert(
            ButtonGridLocation::MetaRight,
            vector![(0..8).map(|_row_idx| ButtonState::Unused).collect()],
        );

        button_states.insert(
            ButtonGridLocation::MetaBottom,
            (0..8)
                .map(|_col_idx| vector![ButtonState::Unused])
                .collect(),
        );

        App {
            link,
            websocket,
            button_states,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        console_log!("{:?}", msg);

        match msg {
            AppMsg::ButtonPressed(location, coords) => {
                // send button press up to the server
                let msg = ClientMessage::ButtonPressed(location.clone(), coords.clone());

                let packet = bincode::serialize(&msg).expect("bincode::serialize");

                let _ = self.websocket.send_binary(Ok(packet));
            }
            AppMsg::ServerMessage(ServerMessage::ButtonStatesUpdated(updates)) => {
                for (location, coords, state) in updates {
                    let grid = self.button_states.get_mut(&location).unwrap();
                    grid[coords.column_idx][coords.row_idx] = state;
                }
            }
            AppMsg::NoOp => {}
        };
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let link = self.link.to_owned();
        let button_callback_fn = callback_fn(move |(location, coord)| {
            link.send_message(AppMsg::ButtonPressed(location, coord));
        });

        html! {
            <div id="app">
                <div class="row row--top">
                    <ButtonGrid
                        location={ButtonGridLocation::Main}
                        button_states={self.button_states[&ButtonGridLocation::Main].clone()}
                        on_button_press={button_callback_fn.clone()}
                    />
                    <ButtonGrid
                        location={ButtonGridLocation::MetaRight}
                        button_states={self.button_states[&ButtonGridLocation::MetaRight].clone()}
                        on_button_press={button_callback_fn.clone()}
                    />
                </div>
                <div class="row row--bottom">
                    <ButtonGrid
                        location={ButtonGridLocation::MetaBottom}
                        button_states={self.button_states[&ButtonGridLocation::MetaBottom].clone()}
                        on_button_press={button_callback_fn.clone()}
                    />
                </div>
            </div>
        }
    }
}
