use im_rc::{HashMap, Vector};
use yew::prelude::*;

use roller_protocol::control::{ButtonCoordinate, ButtonGridLocation, ButtonState};

use crate::{
    app::ButtonAction,
    button_grid::ButtonGrid,
    pure::{Pure, PureComponent},
};

pub type ButtonsPage = Pure<PureButtonsPage>;

#[derive(Properties, Clone, PartialEq)]
pub struct PureButtonsPage {
    pub button_states: HashMap<ButtonGridLocation, Vector<Vector<(Option<String>, ButtonState)>>>,
    pub on_button_action: Callback<(ButtonGridLocation, ButtonCoordinate, ButtonAction)>,
}

impl PureComponent for PureButtonsPage {
    fn render(&self) -> Html {
        html! {
            <div class="page-contents">
                <div class="row row--top">
                    <ButtonGrid
                        location={ButtonGridLocation::Main}
                        button_states={self.button_states[&ButtonGridLocation::Main].clone()}
                        on_button_action={self.on_button_action.clone()}
                    />
                    <ButtonGrid
                        location={ButtonGridLocation::MetaRight}
                        button_states={self.button_states[&ButtonGridLocation::MetaRight].clone()}
                        on_button_action={self.on_button_action.clone()}
                    />
                </div>
                <div class="row row--bottom">
                    <ButtonGrid
                        location={ButtonGridLocation::MetaBottom}
                        button_states={self.button_states[&ButtonGridLocation::MetaBottom].clone()}
                        on_button_action={self.on_button_action.clone()}
                    />
                </div>
            </div>
        }
    }
}
