use yew::prelude::*;

use crate::{button::Button, ButtonCoordinate, ButtonGridLocation, ButtonState};

pub struct ButtonGrid {
    props: ButtonGridProps,
}

pub enum Msg {}

#[derive(Properties, Clone)]
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

    fn view(&self) -> Html {
        let container_class = if let Some(location) = &self.props.location {
            format!("button-grid button-grid--{}", location.css_name())
        } else {
            "button-grid".to_owned()
        };

        html! {
            <div class={container_class}>
                {(0..self.props.rows).map(|row_idx| html! {
                    <div class="button-grid__row">
                    {(0..self.props.columns).map(|column_idx| html! {
                        <Button
                            coordinate={ButtonCoordinate{ row_idx, column_idx }}
                            state={ButtonState::Unused}
                        />
                    }).collect::<Html>()}
                    </div>
                }).collect::<Html>()}
            </div>
        }
    }
}
