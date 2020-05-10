use wasm_bindgen::JsCast;
use yew::prelude::*;

pub struct Fader {
    link: ComponentLink<Self>,
    props: FaderProps,
}

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
            Msg::NoOp => {
                false
            }
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

        // mouse callback
        let onmousedown_callback = self.link.callback(|evt: MouseEvent| {
            let value = target_touch_height_percent(evt.target().unwrap(), &evt);

            Msg::ValueUpdated(value)
        });

        // touch callbacks
        let ontouchstart_callback = self.link.callback(|evt: TouchEvent| {
            let touch = evt.target_touches().get(0).unwrap();
            let value = target_touch_height_percent(evt.target().unwrap(), &touch);

            Msg::ValueUpdated(value)
        });
        let ontouchend_callback = self.link.callback(move |evt: TouchEvent| {
            let touch = evt.changed_touches().get(0);
            match touch {
                Some(touch) => {
                    let value = target_touch_height_percent(evt.target().unwrap(), &touch);
                    Msg::ValueUpdated(value)
                }
                None => Msg::NoOp,
            }
        });
        let ontouchmove_callback = self.link.callback(move |evt: TouchEvent| {
            let touch = evt.changed_touches().get(0);
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
                onmousedown={onmousedown_callback}
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

struct PageCoords {
    x: f64,
    y: f64,
}
impl From<&web_sys::Touch> for PageCoords {
    fn from(touch: &web_sys::Touch) -> PageCoords {
        PageCoords {
            x: touch.page_x() as f64,
            y: touch.page_y() as f64,
        }
    }
}
impl From<&MouseEvent> for PageCoords {
    fn from(evt: &MouseEvent) -> PageCoords {
        PageCoords {
            x: evt.page_x() as f64,
            y: evt.page_y() as f64,
        }
    }
}

// percent in range 0.0 - 1.0, bottom to top, of how far up the touch event is
fn target_touch_height_percent(target: web_sys::EventTarget, coords: impl Into<PageCoords>) -> f64 {
    let coords = coords.into();
    let bounds = event_target_bounds(target);
    let fader_height = bounds.height() as f64;
    let offset_y = coords.y - bounds.top() as f64;
    clamp(1.0 / fader_height * (fader_height - offset_y))
}
