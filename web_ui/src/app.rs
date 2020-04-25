use yew::prelude::*;

use crate::button::Button;

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
                {(0..8).map(|row_idx| html! {
                    <div>
                    {(0..8).map(|col_idx| html! { <Button row_idx={row_idx} column_idx={col_idx}/> }).collect::<Html>()}
                    </div>
                }).collect::<Html>()}
            </div>
        }
    }
}
