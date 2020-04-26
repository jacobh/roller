use yew::prelude::*;

use crate::{
    button::Button, console_log, utils::callback_fn, ButtonCoordinate, ButtonGridLocation,
    ButtonState,
};

pub struct ButtonGrid {
    props: ButtonGridProps,
}

pub enum Msg {}

#[derive(Properties, Clone, PartialEq)]
pub struct ButtonGridProps {
    pub location: Option<ButtonGridLocation>,
    pub rows: usize,
    pub columns: usize,
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
        let container_class = if let Some(location) = &self.props.location {
            format!("button-grid button-grid--{}", location.css_name())
        } else {
            "button-grid".to_owned()
        };

        let callback = callback_fn(|coord: ButtonCoordinate| {
            console_log!("{}", coord);
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
