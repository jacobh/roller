use yew::prelude::*;

use crate::{
    button::Button, utils::callback_fn, ButtonCoordinate, ButtonGridLocation, ButtonState,
};

pub struct ButtonGrid {
    props: ButtonGridProps,
}

pub enum Msg {}

#[derive(Properties, Clone, PartialEq)]
pub struct ButtonGridProps {
    pub location: ButtonGridLocation,
    pub rows: usize,
    pub columns: usize,
    pub on_button_press: Callback<(ButtonGridLocation, ButtonCoordinate)>,
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
            on_button_press,
            ..
        } = self.props.clone();
        let container_class = format!("button-grid button-grid--{}", location.css_name());

        let callback = callback_fn(move |coord: ButtonCoordinate| {
            on_button_press.emit((location.clone(), coord));
        });

        html! {
            <div class={container_class}>
                {(0..self.props.rows).map(|row_idx| html! {
                    <div class="button-grid__row">
                    {(0..self.props.columns).map(|column_idx| html! {
                        <Button
                            coordinate={ButtonCoordinate{ row_idx, column_idx }}
                            state={ButtonState::Unused}
                            on_press={callback.clone()}
                        />
                    }).collect::<Html>()}
                    </div>
                }).collect::<Html>()}
            </div>
        }
    }
}
