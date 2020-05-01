use im_rc::{vector, HashMap, Vector};
use yew::prelude::*;

use crate::{button_grid::ButtonGrid, console_log, utils::callback_fn};
use roller_protocol::{ButtonCoordinate, ButtonGridLocation, ButtonState};

pub struct App {
    link: ComponentLink<Self>,
    button_states: HashMap<ButtonGridLocation, Vector<Vector<ButtonState>>>,
}

#[derive(Debug)]
pub enum Msg {
    ButtonPressed(ButtonGridLocation, ButtonCoordinate),
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut button_states = HashMap::new();

        button_states.insert(
            ButtonGridLocation::Main,
            (0..8)
                .map(|column_idx| {
                    (0..8)
                        .map(|row_idx| {
                            if column_idx < 4 && row_idx > 1 && row_idx < 6 {
                                ButtonState::Inactive
                            } else {
                                ButtonState::Unused
                            }
                        })
                        .collect()
                })
                .collect(),
        );

        button_states.insert(
            ButtonGridLocation::MetaRight,
            vector![(0..8).map(|_row_idx| ButtonState::Inactive).collect()],
        );

        button_states.insert(
            ButtonGridLocation::MetaBottom,
            (0..8)
                .map(|_col_idx| vector![ButtonState::Inactive])
                .collect(),
        );

        App {
            link,
            button_states,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        console_log!("{:?}", msg);
        match msg {
            Msg::ButtonPressed(location, coords) => {
                let grid = self.button_states.get_mut(&location).unwrap();

                let button_state = &mut grid[coords.column_idx][coords.row_idx];
                let next_button_state = match button_state {
                    ButtonState::Inactive => ButtonState::Active,
                    ButtonState::Deactivated => ButtonState::Active,
                    ButtonState::Active => ButtonState::Inactive,
                    ButtonState::Unused => ButtonState::Unused,
                };

                *button_state = next_button_state;
            }
        };
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let link = self.link.to_owned();
        let button_callback_fn = callback_fn(move |(location, coord)| {
            link.send_message(Msg::ButtonPressed(location, coord));
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
