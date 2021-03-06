use im_rc::Vector;
use yew::prelude::*;

use crate::{
    app::ButtonAction,
    pure::{Pure, PureComponent},
    ui::button::Button,
    utils::callback_fn,
};
use roller_protocol::control::{ButtonCoordinate, ButtonGridLocation, ButtonState};

pub type ButtonGrid = Pure<PureButtonGrid>;

#[derive(Properties, Clone, PartialEq)]
pub struct PureButtonGrid {
    pub location: ButtonGridLocation,
    pub button_states: Vector<Vector<(Option<String>, ButtonState)>>,
    pub on_button_action: Callback<(ButtonGridLocation, ButtonCoordinate, ButtonAction)>,
}

impl PureComponent for PureButtonGrid {
    fn render(&self) -> Html {
        let location = self.location.clone();
        let on_button_action = self.on_button_action.clone();

        let container_class = format!("button-grid button-grid--{}", location.css_name());
        let columns = self.button_states.len();
        let rows = self
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
                            label={get_button_label(&self.button_states, column_idx, row_idx)}
                            state={get_button_state(&self.button_states, column_idx, row_idx)}
                            on_action={callback.clone()}
                        />
                    }).collect::<Html>()}
                    </div>
                }).collect::<Html>()}
            </div>
        }
    }
}

fn get_button_info(
    states: &Vector<Vector<(Option<String>, ButtonState)>>,
    column_idx: usize,
    row_idx: usize,
) -> Option<&(Option<String>, ButtonState)> {
    states.get(column_idx).and_then(|row| row.get(row_idx))
}

fn get_button_label(
    states: &Vector<Vector<(Option<String>, ButtonState)>>,
    column_idx: usize,
    row_idx: usize,
) -> String {
    get_button_info(states, column_idx, row_idx)
        .and_then(|(label, _)| label.clone())
        .unwrap_or_else(|| "".to_string())
}

fn get_button_state(
    states: &Vector<Vector<(Option<String>, ButtonState)>>,
    column_idx: usize,
    row_idx: usize,
) -> ButtonState {
    get_button_info(states, column_idx, row_idx)
        .map(|(_, state)| *state)
        .unwrap_or(ButtonState::Unused)
}
