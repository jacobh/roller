use std::str::FromStr;
use yew::prelude::*;

pub struct Fader {
    link: ComponentLink<Self>,
    props: FaderProps,
}

#[allow(dead_code)]
pub enum Msg {
    ValueUpdated(f64),
    NoOp,
}

#[derive(Properties, Clone, PartialEq)]
pub struct FaderProps {
    #[prop_or_default]
    pub label: Option<String>,
    pub value: f64,
    pub on_update: Callback<f64>,
}

impl Component for Fader {
    type Message = Msg;
    type Properties = FaderProps;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Fader { link, props }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::ValueUpdated(value) => {
                self.props.on_update.emit(value);
                true
            }
            Msg::NoOp => false,
        }
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
        let _label = self.props.label.as_deref().unwrap_or("");
        let fill_style = format!("height: {}%", 100.0 - (self.props.value * 100.0));

        let oninput_callback = self.link.callback(move |evt: InputData| {
            // value 0 - 1000
            let value = f64::from_str(&evt.value).unwrap();
            Msg::ValueUpdated(value / 1000.0)
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
