use yew::prelude::*;

use crate::button_grid::ButtonGrid;

pub struct App {}

pub enum Msg {}

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
            <div>
                <ButtonGrid rows={8} columns={8}/>
                <ButtonGrid rows={8} columns={1}/>
                <ButtonGrid rows={1} columns={8}/>
            </div>
        }
    }
}
