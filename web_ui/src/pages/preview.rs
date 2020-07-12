use im_rc::HashMap;
use itertools::Itertools;
use yew::prelude::*;

use roller_protocol::fixture::{FixtureId, FixtureParams, FixtureState};

use crate::pure::{Pure, PureComponent};

fn sorted_unique<T>(items: impl Iterator<Item = T>) -> Vec<T>
where
    T: PartialOrd + PartialEq,
{
    items.fold(Vec::with_capacity(8), |mut output, a| {
        for (i, b) in output.iter().enumerate() {
            // if a is already in the array, skip it
            if &a == b {
                return output;
            }
            // if a is less than the b, insert a before b
            else if b > &a {
                output.insert(i, a);
                return output;
            }
        }

        // a larger than all existing values, so append it to the end
        output.push(a);
        output
    })
}

fn find_index<T>(vec: &Vec<T>, item: &T) -> Option<usize>
where
    T: PartialEq,
{
    vec.iter()
        .enumerate()
        .filter_map(|(i, x)| if x == item { Some(i) } else { None })
        .nth(0)
}

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

        let sorted_rows: Vec<isize> = sorted_unique(
            fixtures
                .iter()
                .filter_map(|fixture| fixture.params.location.as_ref())
                .map(|location| location.y),
        );

        let sorted_columns: Vec<isize> = sorted_unique(
            fixtures
                .iter()
                .filter_map(|fixture| fixture.params.location.as_ref())
                .map(|location| location.x)
                .unique(),
        );

        let fixture_grid: Vec<Vec<Vec<FixtureRef<'_>>>> = {
            let mut grid: Vec<Vec<_>> = (0..sorted_rows.len())
                .map(|_row_idx| {
                    (0..sorted_columns.len())
                        .map(|_col_idx| Vec::with_capacity(1))
                        .collect()
                })
                .collect();

            for fixture in fixtures.into_iter() {
                if let Some(location) = fixture.params.location.as_ref() {
                    let row_idx = find_index(&sorted_rows, &location.y).unwrap();
                    let col_idx = find_index(&sorted_columns, &location.x).unwrap();

                    grid[row_idx][col_idx].push(fixture);
                }
            }

            grid
        };

        html! {
            <div class="page-contents">
                <h2>{"Fixtures"}</h2>
                <div>
                    {fixture_grid.iter().map(|row| html! {
                        <div class="preview__row">
                        {row.iter().map(|column|
                            if let Some(fixture) = column.first() {
                                html! { <PreviewCell fixture_state={fixture.state.clone()}/> }
                            } else {
                                html! { <div class="preview__cell"></div> }
                            }
                        ).collect::<Html>()}
                        </div>
                    }).collect::<Html>()}
                </div>
            </div>
        }
    }
}

pub type PreviewCell = Pure<PurePreviewCell>;

#[derive(Properties, Clone, PartialEq)]
pub struct PurePreviewCell {
    pub fixture_state: FixtureState,
}
impl PureComponent for PurePreviewCell {
    fn render(&self) -> Html {
        let beam = self.fixture_state.beams.values().nth(0).unwrap();
        let color = beam.color.unwrap_or((0.0, 0.0, 0.0));
        let opacity = self.fixture_state.dimmer * beam.dimmer;

        let fill_style = format!(
            "background-color: rgb({}, {}, {}); opacity: {};",
            color.0 * 255.0,
            color.1 * 255.0,
            color.2 * 255.0,
            opacity
        );

        html! {
            <div class="preview__cell preview__cell--active">
                <div class="preview__cell-fill" style={fill_style}/>
            </div>
        }
    }
}
