use im_rc::{vector, HashMap, OrdMap, Vector};
use yew::{
    format::Binary,
    prelude::*,
    services::websocket::{WebSocketService, WebSocketStatus, WebSocketTask},
    utils::window,
};

use crate::{
    pages::{buttons::ButtonsPage, faders::FadersPage, preview::PreviewPage, Page},
    ui::button::Button,
    utils::callback_fn,
};
use roller_protocol::{
    control::{ButtonCoordinate, ButtonGridLocation, ButtonState, FaderId, InputEvent},
    fixture::{FixtureId, FixtureParams, FixtureState},
    ClientMessage, ServerMessage,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ButtonAction {
    Press,
    Release,
}

type FaderValue = f64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PageType {
    Buttons,
    Faders,
    Preview,
}
impl PageType {
    fn is_buttons(&self) -> bool {
        self == &PageType::Buttons
    }
    fn is_faders(&self) -> bool {
        self == &PageType::Faders
    }
    fn is_preview(&self) -> bool {
        self == &PageType::Preview
    }
}

pub struct App {
    link: ComponentLink<Self>,
    websocket: WebSocketTask,
    button_states: HashMap<ButtonGridLocation, Vector<Vector<(Option<String>, ButtonState)>>>,
    fader_states: OrdMap<FaderId, FaderValue>,
    fixture_states: HashMap<FixtureId, (FixtureParams, Option<FixtureState>)>,
    active_page: PageType,
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
    ActivePageUpdated(PageType),
    NoOp,
}

impl Component for App {
    type Message = AppMsg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let host = window().location().host().unwrap();

        let websocket = WebSocketService::connect_binary(
            &format!("ws://{}/ws", host),
            link.callback(|msg: Binary| match msg {
                Ok(buff) => {
                    let msg =
                        bincode::deserialize::<ServerMessage>(&buff).expect("bincode::deserialize");

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
                .map(|_column_idx| (0..8).map(|_row_idx| (None, ButtonState::Unused)).collect())
                .collect(),
        );

        button_states.insert(
            ButtonGridLocation::MetaRight,
            vector![(0..8).map(|_row_idx| (None, ButtonState::Unused)).collect()],
        );

        button_states.insert(
            ButtonGridLocation::MetaBottom,
            (0..8)
                .map(|_col_idx| vector![(None, ButtonState::Unused)])
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
            fixture_states: HashMap::new(),
            active_page: PageType::Buttons,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        // console_log!("{:?}", msg);

        match msg {
            AppMsg::ButtonPressed(location, coords) => {
                self.send_client_message(ClientMessage::Input(InputEvent::ButtonPressed(
                    location, coords,
                )));
            }
            AppMsg::ButtonReleased(location, coords) => {
                self.send_client_message(ClientMessage::Input(InputEvent::ButtonReleased(
                    location, coords,
                )));
            }
            AppMsg::FaderValueUpdated(fader_id, fader_value) => {
                self.fader_states.insert(fader_id, fader_value);
                self.send_client_message(ClientMessage::Input(InputEvent::FaderUpdated(
                    fader_id,
                    fader_value,
                )));
            }
            AppMsg::ServerMessage(ServerMessage::ButtonStatesUpdated(updates)) => {
                for (location, coords, state) in updates {
                    let grid = self.button_states.get_mut(&location).unwrap();
                    grid[coords.column_idx][coords.row_idx].1 = state;
                }
            }
            AppMsg::ServerMessage(ServerMessage::ButtonLabelsUpdated(updates)) => {
                for (location, coords, label) in updates {
                    let grid = self.button_states.get_mut(&location).unwrap();
                    grid[coords.column_idx][coords.row_idx].0 = Some(label);
                }
            }
            AppMsg::ServerMessage(ServerMessage::FixtureParamsUpdated(updates)) => {
                for (fixture_id, fixture_params1) in updates {
                    let fixture_params2 = fixture_params1.clone();

                    self.fixture_states
                        .entry(fixture_id)
                        .or_insert((fixture_params1, None))
                        .0 = fixture_params2;
                }
            }
            AppMsg::ServerMessage(ServerMessage::FixtureStatesUpdated(updates)) => {
                for (fixture_id, fixture_state) in updates {
                    if let Some((_params, state)) = self.fixture_states.get_mut(&fixture_id) {
                        *state = Some(fixture_state)
                    }
                }
            }
            AppMsg::ActivePageUpdated(page_type) => {
                self.active_page = page_type;
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
        let button_callback_fn = callback_fn(move |(location, coord, action)| {
            link.send_message(match action {
                ButtonAction::Press => AppMsg::ButtonPressed(location, coord),
                ButtonAction::Release => AppMsg::ButtonReleased(location, coord),
            })
        });

        let link = self.link.to_owned();
        let fader_button_callback_fn = callback_fn(move |(page_type, action)| {
            if action == ButtonAction::Press {
                link.send_message(AppMsg::ActivePageUpdated(page_type))
            }
        });

        let link = self.link.to_owned();

        html! {
            <div id="app">
                <div class="mode-select">
                    <Button<PageType>
                        id={PageType::Buttons}
                        label={"Buttons"}
                        state={if self.active_page.is_buttons() {ButtonState::Active} else {ButtonState::Inactive}}
                        on_action={fader_button_callback_fn.clone()}
                    />
                    <Button<PageType>
                        id={PageType::Faders}
                        label={"Faders"}
                        state={if self.active_page.is_faders() {ButtonState::Active} else {ButtonState::Inactive}}
                        on_action={fader_button_callback_fn.clone()}
                    />
                    <Button<PageType>
                        id={PageType::Preview}
                        label={"Preview"}
                        state={if self.active_page.is_preview() {ButtonState::Active} else {ButtonState::Inactive}}
                        on_action={fader_button_callback_fn.clone()}
                    />
                </div>
                <Page active={self.active_page.is_buttons()}>
                    <ButtonsPage
                        button_states={self.button_states.clone()}
                        on_button_action={button_callback_fn}
                    />
                </Page>
                <Page active={self.active_page.is_faders()}>
                    <FadersPage
                        fader_states={self.fader_states.clone()}
                        on_fader_update={
                            callback_fn(move |(fader_id, value)| {
                                link.send_message(AppMsg::FaderValueUpdated(fader_id, value))
                            })
                        }
                    />
                </Page>
                <Page active={self.active_page.is_preview()}>
                    <PreviewPage fixture_states={self.fixture_states.clone()} />
                </Page>
            </div>
        }
    }
}
