use wasm_bindgen::JsCast;
use yew::prelude::*;

pub struct Fader {
    link: ComponentLink<Self>,
    props: FaderProps,
    active_touch_id: Option<i32>,
}

pub enum Msg {
    TouchStart(i32, f64),
    TouchEnd(f64),
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
            active_touch_id: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::TouchStart(touch_id, value) => {
                self.active_touch_id = Some(touch_id);
                self.props.on_update.emit(value);
            }
            Msg::TouchEnd(value) => {
                self.active_touch_id = None;
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
        let active_touch_id = self.active_touch_id;
        let _label = self.props.label.as_deref().unwrap_or("");
        let fill_style = format!("height: {}%", 100.0 - (self.props.value * 100.0));

        // touch callbacks
        let ontouchstart_callback = self.link.callback(|evt: TouchEvent| {
            let touch = evt.target_touches().get(0).unwrap();
            let value = target_touch_height_percent(evt.target().unwrap(), &touch);

            Msg::TouchStart(touch.identifier(), value)
        });
        let ontouchend_callback = self.link.callback(move |evt: TouchEvent| {
            let touch = active_touch_id.and_then(|id| evt.changed_touches().item(id as u32));
            match touch {
                Some(touch) => {
                    let value = target_touch_height_percent(evt.target().unwrap(), &touch);
                    Msg::TouchEnd(value)
                }
                None => Msg::NoOp,
            }
        });
        let ontouchmove_callback = self.link.callback(move |evt: TouchEvent| {
            let touch = active_touch_id.and_then(|id| evt.changed_touches().item(id as u32));
            match touch {
                Some(touch) => {
                    let value = target_touch_height_percent(evt.target().unwrap(), &touch);
                    Msg::ValueUpdated(value)
                }
                None => Msg::NoOp,
            }
        });

        html! {
            <div
                class="fader"
                ontouchstart={ontouchstart_callback}
                ontouchend={ontouchend_callback}
                ontouchmove={ontouchmove_callback}
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

fn event_target_bounds(target: web_sys::EventTarget) -> web_sys::DomRect {
    let element: web_sys::Element = target.dyn_into().unwrap();
    element.get_bounding_client_rect()
}

// percent in range 0.0 - 1.0, bottom to top, of how far up the touch event is
fn target_touch_height_percent(target: web_sys::EventTarget, touch: &web_sys::Touch) -> f64 {
    let bounds = event_target_bounds(target);
    let fader_height = bounds.height() as f64;
    let offset_y = touch.page_y() as f64 - bounds.top() as f64;
    clamp(1.0 / fader_height * (fader_height - offset_y))
}
