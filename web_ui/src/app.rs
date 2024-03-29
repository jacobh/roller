use im_rc::{vector, HashMap, OrdMap, Vector};
use std::rc::Rc;
use yew::{
    format::Binary,
    prelude::*,
    services::websocket::{WebSocketService, WebSocketStatus, WebSocketTask},
    utils::window,
};

use crate::{
    pages::{
        buttons::ButtonsPage, faders::FadersPage, preview_2d::Preview2dPage,
        preview_3d::Preview3dPage, Page,
    },
    ui::button::Button,
    utils::callback_fn,
};
use roller_protocol::{
    clock::Clock,
    control::{ButtonCoordinate, ButtonGridLocation, ButtonState, FaderId, InputEvent},
    fixture::{FixtureGroupId, FixtureId, FixtureParams, FixtureState},
    lighting_engine::FixtureGroupState,
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
    Preview2d,
    Preview3d,
}
impl PageType {
    fn is_buttons(&self) -> bool {
        self == &PageType::Buttons
    }
    fn is_faders(&self) -> bool {
        self == &PageType::Faders
    }
    fn is_preview_2d(&self) -> bool {
        self == &PageType::Preview2d
    }
    fn is_preview_3d(&self) -> bool {
        self == &PageType::Preview3d
    }
}

pub struct App {
    link: ComponentLink<Self>,
    websocket: WebSocketTask,
    button_states: HashMap<ButtonGridLocation, Vector<Vector<(Option<String>, ButtonState)>>>,
    fader_states: OrdMap<FaderId, FaderValue>,
    fixture_params: HashMap<FixtureId, FixtureParams>,
    base_fixture_group_state: Rc<FixtureGroupState>,
    fixture_group_states: HashMap<FixtureGroupId, FixtureGroupState>,
    active_page: PageType,
    clock: Rc<Clock>,
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
            fixture_params: HashMap::new(),
            base_fixture_group_state: Rc::new(FixtureGroupState::default()),
            fixture_group_states: HashMap::new(),
            active_page: PageType::Buttons,
            clock: Rc::new(Clock::new(130.0)),
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
            AppMsg::ServerMessage(ServerMessage::ClockUpdated(clock)) => {
                self.clock = Rc::new(clock);
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
                for (fixture_id, fixture_params) in updates {
                    self.fixture_params.insert(fixture_id, fixture_params);
                }
            }
            AppMsg::ServerMessage(ServerMessage::FixtureGroupStatesUpdated(updates)) => {
                for (fixture_group_id, fixture_group_state) in updates {
                    if let Some(fixture_group_id) = fixture_group_id {
                        self.fixture_group_states
                            .insert(fixture_group_id, fixture_group_state);
                    } else {
                        self.base_fixture_group_state = Rc::new(fixture_group_state);
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
                        id={PageType::Preview2d}
                        label={"Preview 2D"}
                        state={if self.active_page.is_preview_2d() {ButtonState::Active} else {ButtonState::Inactive}}
                        on_action={fader_button_callback_fn.clone()}
                    />
                    <Button<PageType>
                        id={PageType::Preview3d}
                        label={"Preview 3D"}
                        state={if self.active_page.is_preview_3d() {ButtonState::Active} else {ButtonState::Inactive}}
                        on_action={fader_button_callback_fn.clone()}
                    />
                </div>
                <Page active={true}>
                {
                    match self.active_page {
                        PageType::Buttons => html! {
                            <ButtonsPage
                                button_states={self.button_states.clone()}
                                on_button_action={button_callback_fn}
                            />
                        },
                        PageType::Faders => html! {
                            <FadersPage
                                fader_states={self.fader_states.clone()}
                                on_fader_update={
                                    callback_fn(move |(fader_id, value)| {
                                        link.send_message(AppMsg::FaderValueUpdated(fader_id, value))
                                    })
                                }
                            />
                        },
                        PageType::Preview2d => html! {
                            <div></div>
                            // TODO
                            // <Preview2dPage fixture_states={self.fixture_states.clone()} />
                        },
                        PageType::Preview3d => html! {
                            <Preview3dPage
                                fixture_params={self.fixture_params.clone()}
                                clock={self.clock.clone()}
                                base_fixture_group_state={self.base_fixture_group_state.clone()}
                                fixture_group_states={self.fixture_group_states.clone()}
                            />
                        }
                    }
                }
                </Page>
            </div>
        }
    }
}
