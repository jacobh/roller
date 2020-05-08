use im_rc::Vector;
use yew::prelude::*;

use crate::{app::ButtonAction, button::Button, utils::callback_fn};
use roller_protocol::{ButtonCoordinate, ButtonGridLocation, ButtonState};

pub struct ButtonGrid {
    props: ButtonGridProps,
}

pub enum Msg {}

#[derive(Properties, Clone, PartialEq)]
pub struct ButtonGridProps {
    pub location: ButtonGridLocation,
    pub button_states: Vector<Vector<ButtonState>>,
    pub on_button_action: Callback<(ButtonGridLocation, ButtonCoordinate, ButtonAction)>,
}

impl Component for ButtonGrid {
    type Message = Msg;
    type Properties = ButtonGridProps;

    fn create(props: Self::Properties, _: ComponentLink<Self>) -> Self {
        ButtonGrid { props }
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
        let ButtonGridProps {
            location,
            on_button_action,
            ..
        } = self.props.clone();
        let container_class = format!("button-grid button-grid--{}", location.css_name());
        let columns = self.props.button_states.len();
        let rows = self
            .props
            .button_states
            .iter()
            .map(|row| row.len())
            .max()
            .unwrap_or(0);

        let callback = callback_fn(move |(coord, action)| {
            on_button_action.emit((location.clone(), coord, action));
        });

        html! {
            <div class={container_class}>
                {(0..rows).map(|row_idx| html! {
                    <div class="button-grid__row">
                    {(0..columns).map(|column_idx| html! {
                        <Button<ButtonCoordinate>
                            id={ButtonCoordinate{ row_idx, column_idx }}
                            label={ButtonCoordinate{ row_idx, column_idx }.to_string()}
                            state={get_button_state(&self.props.button_states, column_idx, row_idx)}
                            on_action={callback.clone()}
                        />
                    }).collect::<Html>()}
                    </div>
                }).collect::<Html>()}
            </div>
        }
    }
}

fn get_button_state(
    states: &Vector<Vector<ButtonState>>,
    column_idx: usize,
    row_idx: usize,
) -> ButtonState {
    states
        .get(column_idx)
        .and_then(|row| row.get(row_idx))
        .cloned()
        .unwrap_or(ButtonState::Unused)
}
