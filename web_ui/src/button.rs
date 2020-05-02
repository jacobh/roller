use yew::prelude::*;

use crate::{app::ButtonAction, utils::callback_fn};
use roller_protocol::{ButtonCoordinate, ButtonState};

pub struct Button {
    props: ButtonProps,
}

pub enum Msg {}

#[derive(Properties, Clone, PartialEq)]
pub struct ButtonProps {
    pub state: ButtonState,
    pub coordinate: ButtonCoordinate,
    pub on_action: Callback<(ButtonCoordinate, ButtonAction)>,
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
            on_action,
            ..
        } = self.props.clone();
        let on_action2 = on_action.clone();

        let onmousedown_callback =
            callback_fn(move |_evt| on_action.emit((coordinate, ButtonAction::Press)));
        let onmouseup_callback =
            callback_fn(move |_evt| on_action2.emit((coordinate, ButtonAction::Release)));

        html! {
            <div class=format!("button {}", self.props.state.css_class()) onmousedown={onmousedown_callback} onmouseup={onmouseup_callback}>
                <span>{self.props.coordinate.to_string()}</span>
            </div>
        }
    }
}
