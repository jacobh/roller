use crate::{
    clock::{offset::offsetted_for_fixture, ClockSnapshot},
    color::Color,
    effect::{self, PixelRangeSet},
    fixture::{Fixture, FixtureGroupId, FixtureParams},
    lighting_engine::FixtureGroupState,
};

pub struct FixtureStateRenderContext<'a> {
    pub base_state: &'a FixtureGroupState,
    pub fixture_group_states: &'a [(&'a FixtureGroupId, &'a FixtureGroupState)],
    pub clock_snapshot: ClockSnapshot,
    pub master_dimmer: f64,
}
pub fn render_fixture_states<'a>(ctx: FixtureStateRenderContext<'a>, fixtures: &mut [Fixture]) {
    let FixtureStateRenderContext {
        master_dimmer,
        clock_snapshot,
        base_state,
        fixture_group_states,
    } = ctx;

    let fixture_params: Vec<FixtureParams> = fixtures
        .iter()
        .map(|fixture| fixture.params.clone())
        .collect::<Vec<_>>();

    let fixture_states = fixtures
        .iter()
        .map(|fixture| {
            let values = fixture
                .params
                .group_id
                .and_then(|group_id| {
                    fixture_group_states
                        .iter()
                        .find(|(id, _)| &group_id == *id)
                        .map(|(_, state)| *state)
                })
                .unwrap_or(base_state);

            let clock_snapshot = clock_snapshot.with_rate(values.clock_rate);

            let effect_dimmer = if fixture.params.dimmer_effects_enabled() {
                values
                    .active_dimmer_effects
                    .iter()
                    .fold(1.0, |dimmer, (effect, rate)| {
                        dimmer
                            * effect::compress(
                                effect.dimmer(&offsetted_for_fixture(
                                    effect.clock_offset.as_ref(),
                                    &clock_snapshot.with_rate(*rate),
                                    &fixture.params,
                                    &fixture_params,
                                )),
                                values.dimmer_effect_intensity(),
                            )
                    })
            } else {
                1.0
            };

            let base_color = values.global_color().to_hsl();
            let secondary_color = values.secondary_color.map(Color::to_hsl);

            let color = if fixture.params.color_effects_enabled() {
                effect::color_intensity(
                    base_color,
                    values
                        .active_color_effects
                        .iter()
                        .fold(base_color, |color, (effect, rate)| {
                            effect.color(
                                color,
                                secondary_color,
                                &offsetted_for_fixture(
                                    effect.clock_offset.as_ref(),
                                    &clock_snapshot.with_rate(*rate),
                                    &fixture.params,
                                    &fixture_params,
                                ),
                            )
                        }),
                    values.color_effect_intensity(),
                )
            } else {
                base_color
            };

            let pixel_range_set: Option<PixelRangeSet> = if fixture.params.pixel_effects_enabled() {
                // TODO only using first active pixel effect
                values
                    .active_pixel_effects
                    .iter()
                    .nth(0)
                    .map(|(effect, rate)| {
                        effect.pixel_range_set(&offsetted_for_fixture(
                            effect.clock_offset.as_ref(),
                            &clock_snapshot.with_rate(*rate),
                            &fixture.params,
                            &fixture_params,
                        ))
                    })
            } else {
                None
            };

            let position = if fixture.params.position_effects_enabled() {
                Some(
                    values
                        .active_position_effects
                        .iter()
                        .map(|(effect, rate)| {
                            effect.position(&offsetted_for_fixture(
                                effect.clock_offset.as_ref(),
                                &clock_snapshot.with_rate(*rate),
                                &fixture.params,
                                &fixture_params,
                            ))
                        })
                        .fold(
                            values.base_position().for_fixture(&fixture, &fixtures),
                            |position1, position2| position1 + position2,
                        ),
                )
            } else {
                None
            };

            let group_dimmer = values.dimmer;

            let dimmer = master_dimmer * group_dimmer * effect_dimmer;
            (dimmer, color, pixel_range_set, position)
        })
        .collect::<Vec<_>>();

    for (fixture, (dimmer, color, pixel_range, position)) in fixtures.iter_mut().zip(fixture_states)
    {
        fixture.state.set_dimmer(dimmer);
        fixture.state.set_color(color);

        if fixture.params.profile.beam_count() > 1 {
            if let Some(pixel_range) = pixel_range {
                fixture.state.set_beam_dimmers(
                    &pixel_range.pixel_dimmers(fixture.params.profile.beam_count()),
                )
            } else {
                // If there's no active pixel effect, reset pixels
                fixture.state.set_all_beam_dimmers(1.0);
            }
        }

        if let Some(position) = position {
            fixture.state.set_position(position);
        }
    }
}
