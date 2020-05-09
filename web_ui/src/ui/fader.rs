use yew::prelude::*;

use crate::utils::callback_fn;

pub struct Fader {
    props: FaderProps,
}

pub enum Msg {}

#[derive(Properties, Clone, PartialEq)]
pub struct FaderProps {
    #[prop_or_default]
    pub label: Option<String>,
    pub value: f64,
    #[prop_or_default]
    pub on_update: Option<Callback<f64>>,
}

impl Component for Fader {
    type Message = Msg;
    type Properties = FaderProps;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Fader { props }
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
        let FaderProps { value, on_update, .. } = self.props.clone();

        let _label = self.props.label.as_deref().unwrap_or("");
        let fill_style = format!("height: {}%", value * 100.0);

        html! {
            <div class="fader">
                <div class="fader__fill" style={fill_style}></div>
            </div>
        }
    }
}
