use im_rc::OrdMap;
use yew::prelude::*;

use roller_protocol::control::fader::FaderId;

use crate::{
    pure::{Pure, PureComponent},
    ui::fader::Fader,
    utils::callback_fn,
};

type FaderValue = f64;

pub type FadersPage = Pure<PureFadersPage>;

#[derive(Properties, Clone, PartialEq)]
pub struct PureFadersPage {
    pub fader_states: OrdMap<FaderId, FaderValue>,
    pub on_fader_update: Callback<(FaderId, FaderValue)>,
}
impl PureComponent for PureFadersPage {
    fn render(&self) -> Html {
        html! {
            <div class="fader-overlay fader-overlay--open">
                {self.fader_states
                    .clone()
                    .into_iter()
                    .map(|(fader_id, fader_value)| {
                        let on_fader_update = self.on_fader_update.clone();
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
