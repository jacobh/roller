use crate::{
    clock::{offset::offsetted_for_fixture, ClockSnapshot},
    color::Color,
    effect,
    fixture::{FixtureGroupId, FixtureParams, FixtureState},
    lighting_engine::FixtureGroupState,
};

pub struct FixtureStateRenderContext<'a> {
    pub base_state: &'a FixtureGroupState,
    pub fixture_group_states: &'a [(&'a FixtureGroupId, &'a FixtureGroupState)],
    pub clock_snapshot: ClockSnapshot,
    pub master_dimmer: f64,
}
pub fn render_fixture_states<'a>(
    ctx: FixtureStateRenderContext<'_>,
    fixture_params: &[&'a FixtureParams],
) -> Vec<(&'a FixtureParams, FixtureState)> {
    let FixtureStateRenderContext {
        master_dimmer,
        clock_snapshot,
        base_state,
        fixture_group_states,
    } = ctx;

    fixture_params
        .iter()
        .map(|params| {
            let mut state = FixtureState::new(&params.profile);

            let group_state = params
                .group_id
                .and_then(|group_id| {
                    fixture_group_states
                        .iter()
                        .find(|(id, _)| &group_id == *id)
                        .map(|(_, state)| *state)
                })
                .unwrap_or(base_state);

            let clock_snapshot = clock_snapshot.with_rate(group_state.clock_rate);

            let effect_dimmer = if params.dimmer_effects_enabled() {
                group_state
                    .active_dimmer_effects
                    .iter()
                    .fold(1.0, |dimmer, (effect, rate)| {
                        dimmer
                            * effect::compress(
                                effect.dimmer(&offsetted_for_fixture(
                                    effect.clock_offset.as_ref(),
                                    &clock_snapshot.with_rate(*rate),
                                    &params,
                                    &fixture_params,
                                )),
                                group_state.dimmer_effect_intensity(),
                            )
                    })
            } else {
                1.0
            };

            let base_color = group_state.global_color().to_hsl();
            let secondary_color = group_state.secondary_color.map(Color::to_hsl);

            let color = if params.color_effects_enabled() {
                effect::color_intensity(
                    base_color,
                    group_state.active_color_effects.iter().fold(
                        base_color,
                        |color, (effect, rate)| {
                            effect.color(
                                color,
                                secondary_color,
                                &offsetted_for_fixture(
                                    effect.clock_offset.as_ref(),
                                    &clock_snapshot.with_rate(*rate),
                                    &params,
                                    &fixture_params,
                                ),
                            )
                        },
                    ),
                    group_state.color_effect_intensity(),
                )
            } else {
                base_color
            };
            state.set_color(color);

            if params.pixel_effects_enabled() {
                // TODO only using first active pixel effect
                let pixel_range =
                    group_state
                        .active_pixel_effects
                        .iter()
                        .nth(0)
                        .map(|(effect, rate)| {
                            effect.pixel_range_set(&offsetted_for_fixture(
                                effect.clock_offset.as_ref(),
                                &clock_snapshot.with_rate(*rate),
                                &params,
                                &fixture_params,
                            ))
                        });

                if params.profile.beam_count() > 1 {
                    if let Some(pixel_range) = pixel_range {
                        state.set_beam_dimmers(
                            &pixel_range.pixel_dimmers(params.profile.beam_count()),
                        )
                    } else {
                        // If there's no active pixel effect, reset pixels
                        state.set_all_beam_dimmers(1.0);
                    }
                }
            }

            if params.position_effects_enabled() {
                let position = group_state
                    .active_position_effects
                    .iter()
                    .map(|(effect, rate)| {
                        effect.position(&offsetted_for_fixture(
                            effect.clock_offset.as_ref(),
                            &clock_snapshot.with_rate(*rate),
                            &params,
                            &fixture_params,
                        ))
                    })
                    .fold(
                        group_state
                            .base_position()
                            .for_fixture(&params, &fixture_params),
                        |position1, position2| position1 + position2,
                    );
                state.set_position(position);
            }

            let group_dimmer = group_state.dimmer;

            let dimmer = master_dimmer * group_dimmer * effect_dimmer;
            state.set_dimmer(dimmer);

            (*params, state)
        })
        .collect()
}
