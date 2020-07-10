use im_rc::{HashMap, Vector};
use yew::prelude::*;

use roller_protocol::control::{ButtonCoordinate, ButtonGridLocation, ButtonState};

use crate::{app::ButtonAction, button_grid::ButtonGrid};

pub struct ButtonsPage {
    props: ButtonsPageProps,
}

#[derive(Properties, Clone, PartialEq)]
pub struct ButtonsPageProps {
    pub button_states: HashMap<ButtonGridLocation, Vector<Vector<(Option<String>, ButtonState)>>>,
    pub on_button_action: Callback<(ButtonGridLocation, ButtonCoordinate, ButtonAction)>,
}

impl Component for ButtonsPage {
    type Message = ();
    type Properties = ButtonsPageProps;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        ButtonsPage { props }
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
            <div>
                <div class="row row--top">
                    <ButtonGrid
                        location={ButtonGridLocation::Main}
                        button_states={self.props.button_states[&ButtonGridLocation::Main].clone()}
                        on_button_action={self.props.on_button_action.clone()}
                    />
                    <ButtonGrid
                        location={ButtonGridLocation::MetaRight}
                        button_states={self.props.button_states[&ButtonGridLocation::MetaRight].clone()}
                        on_button_action={self.props.on_button_action.clone()}
                    />
                </div>
                <div class="row row--bottom">
                    <ButtonGrid
                        location={ButtonGridLocation::MetaBottom}
                        button_states={self.props.button_states[&ButtonGridLocation::MetaBottom].clone()}
                        on_button_action={self.props.on_button_action.clone()}
                    />
                </div>
            </div>
        }
    }
}
