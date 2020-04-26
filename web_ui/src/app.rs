use yew::prelude::*;

use crate::{
    button_grid::ButtonGrid, console_log, utils::callback_fn, ButtonCoordinate, ButtonGridLocation,
};

pub struct App {}

pub enum Msg {
    ButtonPressed(ButtonGridLocation, ButtonCoordinate),
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, _: ComponentLink<Self>) -> Self {
        App {}
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
                        on_button_press={button_callback_fn.clone()}
                        rows={8}
                        columns={8}
                    />
                    <ButtonGrid
                        location={ButtonGridLocation::MetaRight}
                        on_button_press={button_callback_fn.clone()}
                        rows={8}
                        columns={1}
                    />
                </div>
                <div class="row row--bottom">
                    <ButtonGrid
                        location={ButtonGridLocation::MetaBottom}
                        on_button_press={button_callback_fn.clone()}
                        rows={1}
                        columns={8}
                    />
                </div>
            </div>
        }
    }
}
