use std::str::FromStr;
use yew::prelude::*;

use crate::{
    pure::{Pure, PureComponent},
    utils::callback_fn,
};

pub type Fader = Pure<PureFader>;

#[derive(Properties, Clone, PartialEq)]
pub struct PureFader {
    #[prop_or_default]
    pub label: Option<String>,
    pub value: f64,
    pub on_update: Callback<f64>,
}

impl PureComponent for PureFader {
    fn render(&self) -> Html {
        let _label = self.label.as_deref().unwrap_or("");
        let fill_style = format!("height: {}%", 100.0 - (self.value * 100.0));

        let on_update = self.on_update.clone();
        let oninput_callback = callback_fn(move |evt: InputData| {
            // value 0 - 1000
            let value = f64::from_str(&evt.value).unwrap();
            on_update.emit(value / 1000.0);
        });

        html! {
            <div
                class="fader"
            >
                <input
                    class="fader__range-input"
                    orient="vertical"
                    type="range"
                    min="0"
                    max="1000"
                    oninput={oninput_callback}
                />
                <div class="fader__fill" style={fill_style}></div>
            </div>
        }
    }
}
