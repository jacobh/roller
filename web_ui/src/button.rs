use yew::prelude::*;

pub struct Button {
    props: ButtonProps,
}

pub enum Msg {}

#[derive(Properties, Clone)]
pub struct ButtonProps {
    pub column_idx: usize,
    pub row_idx: usize,
}

impl Component for Button {
    type Message = Msg;
    type Properties = ButtonProps;

    fn create(props: Self::Properties, _: ComponentLink<Self>) -> Self {
        Button { props }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        true
    }

    fn view(&self) -> Html {
        html! {
            <span>{"("}{self.props.column_idx}{", "}{self.props.row_idx}{")"}</span>
        }
    }
}
