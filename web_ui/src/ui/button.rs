use yew::prelude::*;

use crate::{app::ButtonAction, utils::callback_fn};
use roller_protocol::control::ButtonState;

pub struct Button<T>
where
    T: Clone,
{
    props: ButtonProps<T>,
}

pub enum Msg {}

#[derive(Properties, Clone, PartialEq)]
pub struct ButtonProps<T>
where
    T: Clone,
{
    pub id: T,
    #[prop_or_default]
    pub label: Option<String>,
    #[prop_or_default]
    pub state: ButtonState,
    #[prop_or_default]
    pub on_action: Option<Callback<(T, ButtonAction)>>,
}

impl<T> Component for Button<T>
where
    T: 'static + Clone + Copy + PartialEq,
{
    type Message = Msg;
    type Properties = ButtonProps<T>;

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
        let ButtonProps { id, on_action, .. } = self.props.clone();
        let on_action2 = on_action.clone();

        let onmousedown_callback = callback_fn(move |_evt| {
            if let Some(on_action) = &on_action {
                on_action.emit((id.clone(), ButtonAction::Press));
            }
        });

        let onmouseup_callback = callback_fn(move |_evt| {
            if let Some(on_action) = &on_action2 {
                on_action.emit((id.clone(), ButtonAction::Release));
            }
        });

        let label = self.props.label.as_deref().unwrap_or("");

        html! {
            <div
                class=format!("button {}", self.props.state.css_class())
                onmousedown={onmousedown_callback}
                onmouseup={onmouseup_callback}
            >
                <span>{label}</span>
            </div>
        }
    }
}
