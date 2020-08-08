use yew::prelude::*;

pub mod buttons;
pub mod faders;
pub mod preview_2d;
pub mod preview_3d;

pub struct Page {
    props: PageProps,
}

#[derive(Properties, Clone, PartialEq)]
pub struct PageProps {
    pub children: Children,
    pub active: bool,
}

impl Component for Page {
    type Message = ();
    type Properties = PageProps;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Page { props }
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
        if self.props.active {
            html! {
                <div class="page page--active">
                    {self.props.children.clone()}
                </div>
            }
        } else {
            html! {
                <div class="page page--inactive"></div>
            }
        }
    }
}
