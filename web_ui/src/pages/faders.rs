use im_rc::OrdMap;
use yew::prelude::*;

use roller_protocol::control::FaderId;

use crate::{pages::Page, ui::fader::Fader, utils::callback_fn};

type FaderValue = f64;

pub struct FadersPage {
    props: FadersPageProps,
}

#[derive(Properties, Clone, PartialEq)]
pub struct FadersPageProps {
    pub fader_states: OrdMap<FaderId, FaderValue>,
    pub on_fader_update: Callback<(FaderId, FaderValue)>,
}

impl Component for FadersPage {
    type Message = ();
    type Properties = FadersPageProps;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        FadersPage { props }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
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
        html! {
            <div class="fader-overlay fader-overlay--open">
                {self.props.fader_states
                    .clone()
                    .into_iter()
                    .map(|(fader_id, fader_value)| {
                        let on_fader_update = self.props.on_fader_update.clone();
                        let on_update_fn = callback_fn(move |value| {
                            on_fader_update.emit((fader_id, value));
                        });
                        html! {
                            <Fader
                                value={fader_value}
                                on_update={on_update_fn}
                            />
                        }
                    })
                    .collect::<Html>()}
            </div>
        }
    }
}
