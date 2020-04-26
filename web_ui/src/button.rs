use yew::prelude::*;

use crate::{ButtonCoordinate, ButtonState};

pub struct Button {
    props: ButtonProps,
}

pub enum Msg {}

#[derive(Properties, Clone)]
pub struct ButtonProps {
    pub state: ButtonState,
    pub coordinate: Option<ButtonCoordinate>,
}

impl Component for Button {
    type Message = Msg;
    type Properties = ButtonProps;

    fn create(props: Self::Properties, _: ComponentLink<Self>) -> Self {
        Button { props }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        true
    }

    fn view(&self) -> Html {
        let label = self
            .props
            .coordinate
            .as_ref()
            .map(|coords| coords.to_string())
            .unwrap_or("".to_string());

        html! {
            <div class=format!("button {}", self.props.state.css_class())>
                <span>{label}</span>
            </div>
        }
    }
}
