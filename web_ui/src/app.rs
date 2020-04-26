use yew::prelude::*;

use crate::{button_grid::ButtonGrid, ButtonGridLocation, ButtonCoordinate};

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

    fn view(&self) -> Html {
        html! {
            <div id="app">
                <div class="row row--top">
                    <ButtonGrid location={ButtonGridLocation::Main} rows={8} columns={8}/>
                    <ButtonGrid location={ButtonGridLocation::MetaRight} rows={8} columns={1}/>
                </div>
                <div class="row row--bottom">
                    <ButtonGrid location={ButtonGridLocation::MetaBottom} rows={1} columns={8}/>
                </div>
            </div>
        }
    }
}
