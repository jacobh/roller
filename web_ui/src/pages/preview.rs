use im_rc::HashMap;
use itertools::Itertools;
use yew::prelude::*;

use roller_protocol::fixture::{FixtureId, FixtureParams, FixtureState};

use crate::pure::{Pure, PureComponent};

struct FixtureRef<'a> {
    id: &'a FixtureId,
    params: &'a FixtureParams,
    state: &'a FixtureState,
}

impl<'a> From<(&'a FixtureId, &'a FixtureParams, &'a FixtureState)> for FixtureRef<'a> {
    fn from(
        (id, params, state): (&'a FixtureId, &'a FixtureParams, &'a FixtureState),
    ) -> FixtureRef<'a> {
        FixtureRef { id, params, state }
    }
}

pub type PreviewPage = Pure<PurePreviewPage>;

#[derive(Properties, Clone, PartialEq)]
pub struct PurePreviewPage {
    pub fixture_states: HashMap<FixtureId, (FixtureParams, Option<FixtureState>)>,
}
impl PureComponent for PurePreviewPage {
    fn render(&self) -> Html {
        let fixtures: Vec<FixtureRef<'_>> = self
            .fixture_states
            .iter()
            .filter_map(|(fixture_id, (params, state))| match state {
                Some(state) => Some((fixture_id, params, state)),
                None => None,
            })
            .map(FixtureRef::from)
            .collect();

        let rows = fixtures
            .iter()
            .filter_map(|fixture| fixture.params.location.as_ref())
            .map(|location| location.y)
            .unique()
            .count();
        let columns = fixtures
            .iter()
            .filter_map(|fixture| fixture.params.location.as_ref())
            .map(|location| location.x)
            .unique()
            .count();

        html! {
            <div class="page-contents">
                <h2>{"Fixtures"}</h2>
                <div>
                    {(0..rows).map(|row_idx| html! {
                        <div class="preview__row">
                        {(0..columns).map(|column_idx| html! {
                            <div id={format!("preview__cell-{}-{}", column_idx, row_idx)} class="preview__cell"></div>
                        }).collect::<Html>()}
                        </div>
                    }).collect::<Html>()}
                </div>
            </div>
        }
    }
}
