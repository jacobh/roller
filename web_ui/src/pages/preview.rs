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
                    {(0..sorted_rows.len()).map(|row_idx| html! {
                        <div class="preview__row">
                        {(0..sorted_columns.len()).map(|column_idx| html! {
                            <div id={format!("preview__cell-{}-{}", column_idx, row_idx)} class="preview__cell"></div>
                        }).collect::<Html>()}
                        </div>
                    }).collect::<Html>()}
                </div>
            </div>
        }
    }
}
