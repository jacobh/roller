use std::collections::HashMap;
use yew::prelude::*;

use crate::{
    button_grid::ButtonGrid, console_log, utils::callback_fn, ButtonCoordinate, ButtonGridLocation,
    ButtonState,
};

pub struct App {
    button_states: HashMap<ButtonGridLocation, Vec<Vec<ButtonState>>>,
}

pub enum Msg {
    ButtonPressed(ButtonGridLocation, ButtonCoordinate),
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, _: ComponentLink<Self>) -> Self {
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
            vec![(0..8).map(|_row_idx| ButtonState::Inactive).collect()],
        );

        button_states.insert(
            ButtonGridLocation::MetaBottom,
            (0..8).map(|_col_idx| vec![ButtonState::Inactive]).collect(),
        );

        App { button_states }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let button_callback_fn = callback_fn(|(location, coord)| {
            console_log!("{:?}: {}", location, coord);
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
