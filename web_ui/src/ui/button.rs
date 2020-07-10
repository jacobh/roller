use yew::prelude::*;

use crate::{
    app::ButtonAction,
    pure::{Pure, PureComponent},
    utils::callback_fn,
};
use roller_protocol::control::ButtonState;

pub type Button<T> = Pure<PureButton<T>>;

#[derive(Properties, Clone, PartialEq)]
pub struct PureButton<T>
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

impl<T> PureComponent for PureButton<T>
where
    T: 'static + Clone + Copy + PartialEq,
{
    fn render(&self) -> Html {
        let id = self.id.clone();
        let on_action = self.on_action.clone();
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

        let label = self.label.as_deref().unwrap_or("");

        html! {
            <div
                class=format!("button {}", self.state.css_class())
                onmousedown={onmousedown_callback}
                onmouseup={onmouseup_callback}
            >
                <span>{label}</span>
            </div>
        }
    }
}
