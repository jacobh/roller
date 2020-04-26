use yew::prelude::*;

use crate::{ButtonCoordinate, ButtonState};

pub struct Button {
    link: ComponentLink<Self>,
    props: ButtonProps,
}

pub enum Msg {}

#[derive(Properties, Clone, PartialEq)]
pub struct ButtonProps {
    pub state: ButtonState,
    pub coordinate: ButtonCoordinate,
}

impl Component for Button {
    type Message = Msg;
    type Properties = ButtonProps;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Button { link, props }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if self.props != props {
            self.props = props;
            true
        } else {
            false
        }
    }

    fn view(&self) -> Html {
        let label = self.props.coordinate.to_string();

        html! {
            <div class=format!("button {}", self.props.state.css_class()) >
                <span>{label}</span>
            </div>
        }
    }
}
