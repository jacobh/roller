use std::rc::Rc;
use yew::prelude::*;

use crate::{ButtonCoordinate, ButtonState};

pub struct Button {
    props: ButtonProps,
}

pub enum Msg {}

#[derive(Properties, Clone, PartialEq)]
pub struct ButtonProps {
    pub state: ButtonState,
    pub coordinate: ButtonCoordinate,
    pub on_press: Callback<ButtonCoordinate>,
}

impl Component for Button {
    type Message = Msg;
    type Properties = ButtonProps;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Button { props }
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
        let ButtonProps {
            coordinate,
            on_press,
            ..
        } = self.props.clone();
        let click_callback =
            Callback::Callback(Rc::new(move |_evt| on_press.emit(coordinate.clone())));

        html! {
            <div class=format!("button {}", self.props.state.css_class()) onclick={click_callback}>
                <span>{self.props.coordinate.to_string()}</span>
            </div>
        }
    }
}
