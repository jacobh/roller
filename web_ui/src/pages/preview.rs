use im_rc::HashMap;
use yew::prelude::*;

use roller_protocol::fixture::{FixtureId, FixtureParams, FixtureState};

use crate::{
    pure::{Pure, PureComponent},
};

pub type PreviewPage = Pure<PurePreviewPage>;

#[derive(Properties, Clone, PartialEq)]
pub struct PurePreviewPage {
    pub fixture_states: HashMap<FixtureId, (FixtureParams, Option<FixtureState>)>,
}
impl PureComponent for PurePreviewPage {
    fn render(&self) -> Html {
        html! {
            <div class="page-contents">
                <h2>{"Preview coming soon"}</h2>
            </div>
        }
    }
}
