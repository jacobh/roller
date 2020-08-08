use im_rc::HashMap;
use itertools::Itertools;
use yew::prelude::*;

use roller_protocol::fixture::{FixtureId, FixtureParams, FixtureState};

use crate::pure::{Pure, PureComponent};

pub type Preview3dPage = Pure<PurePreview3dPage>;

#[derive(Properties, Clone, PartialEq)]
pub struct PurePreview3dPage {
    pub fixture_states: HashMap<FixtureId, (FixtureParams, Option<FixtureState>)>,
}
impl PureComponent for PurePreview3dPage {
    fn render(&self) -> Html {
        html! {
            <div class="page-contents">
                <h2>{"Fixtures 3D"}</h2>
                <div></div>
            </div>
        }
    }
}
