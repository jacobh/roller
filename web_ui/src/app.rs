use im_rc::{vector, HashMap, OrdMap, Vector};
use yew::{
    format::Binary,
    prelude::*,
    services::websocket::{WebSocketService, WebSocketStatus, WebSocketTask},
    utils::window,
};

use crate::{
    button_grid::ButtonGrid,
    console_log,
    ui::{button::Button, fader::Fader},
    utils::callback_fn,
};
use roller_protocol::{
    ButtonCoordinate, ButtonGridLocation, ButtonState, ClientMessage, FaderId, ServerMessage,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ButtonAction {
    Press,
    Release,
}

type FaderValue = f64;

pub struct App {
    link: ComponentLink<Self>,
    websocket: WebSocketTask,
    button_states: HashMap<ButtonGridLocation, Vector<Vector<ButtonState>>>,
    fader_states: OrdMap<FaderId, FaderValue>,
    fader_overlay_open: bool,
}

impl App {
    fn send_client_message(&mut self, message: ClientMessage) {
        let packet = bincode::serialize(&message).expect("bincode::serialize");
        self.websocket.send_binary(Ok(packet))
    }
}

#[derive(Debug)]
pub enum AppMsg {
    ButtonPressed(ButtonGridLocation, ButtonCoordinate),
    ButtonReleased(ButtonGridLocation, ButtonCoordinate),
    FaderValueUpdated(FaderId, FaderValue),
    ServerMessage(ServerMessage),
    FaderOverlayToggled,
    NoOp,
}

impl Component for App {
    type Message = AppMsg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let host = window().location().host().unwrap();
        let mut websocket = WebSocketService::new();

        let websocket = websocket
            .connect_binary(
                &format!("ws://{}/ws", host),
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

        let fader_states = (0..9)
            .map(|fader_idx| (FaderId::new(fader_idx), 0.667))
            .collect();

        App {
            link,
            websocket,
            button_states,
            fader_states,
            fader_overlay_open: false,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        // console_log!("{:?}", msg);

        match msg {
            AppMsg::ButtonPressed(location, coords) => {
                self.send_client_message(ClientMessage::ButtonPressed(location, coords));
            }
            AppMsg::ButtonReleased(location, coords) => {
                self.send_client_message(ClientMessage::ButtonReleased(location, coords));
            }
            AppMsg::FaderValueUpdated(fader_id, fader_value) => {
                self.fader_states.insert(fader_id, fader_value);
                self.send_client_message(ClientMessage::FaderUpdated(fader_id, fader_value));
            }
            AppMsg::ServerMessage(ServerMessage::ButtonStatesUpdated(updates)) => {
                for (location, coords, state) in updates {
                    let grid = self.button_states.get_mut(&location).unwrap();
                    grid[coords.column_idx][coords.row_idx] = state;
                }
            }
            AppMsg::FaderOverlayToggled => {
                self.fader_overlay_open = !self.fader_overlay_open;
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
        let button_callback_fn = callback_fn(move |(location, coord, action)| match action {
            ButtonAction::Press => {
                link.send_message(AppMsg::ButtonPressed(location, coord));
            }
            ButtonAction::Release => {
                link.send_message(AppMsg::ButtonReleased(location, coord));
            }
        });

        let link = self.link.to_owned();
        let fader_button_callback_fn = callback_fn(move |((), action)| {
            if action == ButtonAction::Press {
                link.send_message(AppMsg::FaderOverlayToggled)
            }
        });

        let link = self.link.to_owned();

        html! {
            <div id="app">
                <div class="row row--top">
                    <ButtonGrid
                        location={ButtonGridLocation::Main}
                        button_states={self.button_states[&ButtonGridLocation::Main].clone()}
                        on_button_action={button_callback_fn.clone()}
                    />
                    <ButtonGrid
                        location={ButtonGridLocation::MetaRight}
                        button_states={self.button_states[&ButtonGridLocation::MetaRight].clone()}
                        on_button_action={button_callback_fn.clone()}
                    />
                </div>
                <div class="row row--bottom">
                    <div class="mode-select">
                        <Button<()>
                            id={()}
                            label={"Faders"}
                            state={if self.fader_overlay_open {ButtonState::Active} else {ButtonState::Unused}}
                            on_action={fader_button_callback_fn}
                        />
                    </div>
                    <ButtonGrid
                        location={ButtonGridLocation::MetaBottom}
                        button_states={self.button_states[&ButtonGridLocation::MetaBottom].clone()}
                        on_button_action={button_callback_fn.clone()}
                    />
                </div>
                <div
                    class={format!(
                        "fader-overlay {}",
                        if self.fader_overlay_open {"fader-overlay--open"} else {"fader-overlay--closed"}
                    )}
                >
                {self.fader_states
                    .clone()
                    .into_iter()
                    .map(|(fader_id, fader_value)| {
                        let link = link.clone();
                        let on_update_fn = callback_fn(move |value| {
                            link.send_message(AppMsg::FaderValueUpdated(fader_id, value))
                        });
                        html! {
                            <Fader
                                value={fader_value}
                                on_update={on_update_fn}
                            />
                        }
                    })
                    .collect::<Html>()}
                </div>
            </div>
        }
    }
}
