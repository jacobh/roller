use im_rc::HashMap;
use yew::prelude::*;

use roller_protocol::fixture::{FixtureId, FixtureParams, FixtureState};

use crate::pure::{Pure, PureComponent};

pub type PreviewPage = Pure<PurePreviewPage>;

#[derive(Properties, Clone, PartialEq)]
pub struct PurePreviewPage {
    pub fixture_states: HashMap<FixtureId, (FixtureParams, Option<FixtureState>)>,
}
impl PureComponent for PurePreviewPage {
    fn render(&self) -> Html {
        let fixtures: Vec<Html> = self
            .fixture_states
            .iter()
            .filter_map(|(fixture_id, (params, state))| match state {
                Some(state) => Some((fixture_id, params, state)),
                None => None,
            })
            .map(|(_fixture_id, params, state)| {
                html! {
                    <dl>
                        <dt>{"Profile"}</dt>
                        <dd>{params.profile.label.clone()}</dd>
                        <dt>{"Location"}</dt>
                        <dd>{format!("{:?}", params.location)}</dd>
                        <dt>{"Dimmer"}</dt>
                        <dd>{format!("{:?}", state.dimmer)}</dd>
                    </dl>
                }
            })
            .collect();

        html! {
            <div class="page-contents">
                <h2>{"Fixtures"}</h2>
                {fixtures}
            </div>
        }
    }
}
