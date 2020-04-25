use yew::prelude::*;

use crate::button::Button;

pub struct ButtonGrid {
    props: ButtonGridProps,
}

pub enum Msg {}

#[derive(Properties, Clone)]
pub struct ButtonGridProps {
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
        html! {
            <div>
                {(0..self.props.rows).map(|row_idx| html! {
                    <div>
                    {(0..self.props.columns).map(|col_idx| html! {
                        <Button row_idx={row_idx} column_idx={col_idx}/>
                    }).collect::<Html>()}
                    </div>
                }).collect::<Html>()}
            </div>
        }
    }
}
