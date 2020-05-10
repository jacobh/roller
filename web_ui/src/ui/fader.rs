use wasm_bindgen::JsCast;
use yew::prelude::*;

pub struct Fader {
    link: ComponentLink<Self>,
    props: FaderProps,
    input_active: bool,
}

pub enum Msg {
    MouseDown(f64),
    MouseUp(f64),
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
        Fader {
            link,
            props,
            input_active: false,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::MouseDown(value) => {
                self.input_active = true;
                self.props.on_update.emit(value);
            }
            Msg::MouseUp(value) => {
                self.input_active = false;
                self.props.on_update.emit(value);
            }
            Msg::ValueUpdated(value) => {
                self.props.on_update.emit(value);
            }
            Msg::NoOp => {}
        }
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
        let input_active = self.input_active;
        let _label = self.props.label.as_deref().unwrap_or("");
        let fill_style = format!("height: {}%", 100.0 - (self.props.value * 100.0));

        let onmousedown_callback = self.link.callback(|evt: MouseEvent| {
            let value = mouse_event_to_fader_percent(evt);
            Msg::MouseDown(value)
        });
        let onmouseup_callback = self.link.callback(move |evt: MouseEvent| {
            if input_active {
                let value = mouse_event_to_fader_percent(evt);
                Msg::MouseUp(value)
            } else {
                Msg::NoOp
            }
        });
        let onmousemove_callback = self.link.callback(move |evt: MouseEvent| {
            if input_active {
                let value = mouse_event_to_fader_percent(evt);
                Msg::ValueUpdated(value)
            } else {
                Msg::NoOp
            }
        });

        html! {
            <div
                class="fader"
                onmousedown={onmousedown_callback}
                onmouseup={onmouseup_callback.clone()}
                onmousemove={onmousemove_callback}
                onmouseout={onmouseup_callback}
            >
                <div class="fader__fill" style={fill_style}></div>
            </div>
        }
    }
}

fn clamp(x: f64) -> f64 {
    if x > 1.0 {
        1.0
    } else if x < 0.0 {
        0.0
    } else {
        x
    }
}

fn mouse_event_to_fader_percent(evt: MouseEvent) -> f64 {
    let fader_element: web_sys::HtmlElement = evt.current_target().unwrap().dyn_into().unwrap();

    let fader_height = fader_element.offset_height() as f64;
    let offset_y = evt.offset_y() as f64;
    clamp(1.0 / fader_height * (fader_height - offset_y))
}
