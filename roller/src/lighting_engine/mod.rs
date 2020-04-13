use derive_more::Constructor;
use rustc_hash::{FxHashMap, FxHashSet};
use std::borrow::Cow;
use std::time::Instant;

use crate::{
    clock::{offsetted_for_fixture, Clock, ClockEvent, Rate},
    color::Color,
    control::{
        button::{ButtonGroup, ButtonMapping, MetaButtonAction, PadEvent},
        midi::{MidiMapping, NoteState},
    },
    effect::{
        self, ColorEffect, DimmerEffect, PixelEffect, PixelEffectOverride, PixelRangeSet,
        PositionEffect,
    },
    fixture::Fixture,
    position::BasePosition,
    project::FixtureGroupId,
    utils::FxIndexMap,
};

mod button_states;

pub use button_states::{
    ButtonGroupInfo, ButtonInfo, ButtonStateMap, ButtonStateValue, ButtonStates, FixtureGroupState,
    SceneState, EMPTY_SCENE_STATE,
};

// This is just for the case where no buttons have been activated yet
lazy_static::lazy_static! {
    static ref DEFAULT_FIXTURE_GROUP_VALUE: FixtureGroupValue<'static> = FixtureGroupValue::default();
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Constructor)]
pub struct SceneId(usize);

#[derive(Default)]
pub struct FixtureGroupValue<'a> {
    pub dimmer: f64,
    pub clock_rate: Rate,
    pub global_color: Option<Color>,
    pub secondary_color: Option<Color>,
    pub base_position: Option<BasePosition>,
    pub active_dimmer_effects: FxIndexMap<&'a DimmerEffect, Rate>,
    pub active_color_effects: FxIndexMap<&'a ColorEffect, Rate>,
    pub active_pixel_effects: FxIndexMap<&'a PixelEffect, Rate>,
    pub active_position_effects: FxIndexMap<&'a PositionEffect, Rate>,
    pub active_pixel_effect_overrides: FxHashSet<&'a PixelEffectOverride>,
}
impl<'a> FixtureGroupValue<'a> {
    pub fn merge(mut self, other: &FixtureGroupValue<'a>) -> FixtureGroupValue<'a> {
        self.clock_rate = self.clock_rate * other.clock_rate;
        if self.global_color == None {
            self.global_color = other.global_color;
        }
        if self.secondary_color == None {
            self.secondary_color = other.secondary_color;
        }
        if self.base_position == None {
            self.base_position = other.base_position;
        }
        self.active_dimmer_effects
            .extend(other.active_dimmer_effects.iter());
        self.active_color_effects
            .extend(other.active_color_effects.iter());
        self.active_pixel_effects
            .extend(other.active_pixel_effects.iter());
        self.active_pixel_effect_overrides
            .extend(other.active_pixel_effect_overrides.iter());
        self.active_position_effects
            .extend(other.active_position_effects.iter());

        self
    }
    pub fn global_color(&self) -> Color {
        self.global_color.unwrap_or(Color::Violet)
    }
    pub fn base_position(&self) -> BasePosition {
        self.base_position.unwrap_or_default()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ControlMode {
    Shift,
    Normal,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ControlEvent {
    UpdateMasterDimmer(f64),
    UpdateGroupDimmer(FixtureGroupId, f64),
    UpdateDimmerEffectIntensity(f64),
    UpdateColorEffectIntensity(f64),
    UpdateClockRate(Rate),
    SelectScene(SceneId),
    SelectFixtureGroupControl(FixtureGroupId),
    UpdateButton(ButtonGroup, ButtonMapping, NoteState, Instant),
    TapTempo(Instant),
    UpdateControlMode(ControlMode),
}

pub struct EngineState<'a> {
    pub midi_mapping: &'a MidiMapping,
    pub clock: Clock,
    pub master_dimmer: f64,
    pub dimmer_effect_intensity: f64,
    pub color_effect_intensity: f64,
    pub control_mode: ControlMode,
    pub active_scene_id: SceneId,
    pub active_fixture_group_control: Option<FixtureGroupId>,
    pub scene_fixture_group_button_states: FxHashMap<SceneId, SceneState>,
}
impl<'a> EngineState<'a> {
    pub fn new(midi_mapping: &'a MidiMapping) -> EngineState<'a> {
        EngineState {
            midi_mapping,
            clock: Clock::new(128.0),
            master_dimmer: 1.0,
            dimmer_effect_intensity: 0.5,
            color_effect_intensity: 1.0,
            control_mode: ControlMode::Normal,
            active_scene_id: SceneId::new(1),
            active_fixture_group_control: None,
            scene_fixture_group_button_states: FxHashMap::default(),
        }
    }
    fn active_scene_state(&self) -> &SceneState {
        self.scene_fixture_group_button_states
            .get(&self.active_scene_id)
            .unwrap_or_else(|| &*EMPTY_SCENE_STATE)
    }
    fn active_scene_state_mut(&mut self) -> &mut SceneState {
        self.scene_fixture_group_button_states
            .entry(self.active_scene_id)
            .or_default()
    }
    pub fn control_fixture_group_state(&self) -> &FixtureGroupState {
        &self
            .active_scene_state()
            .fixture_group_state(self.active_fixture_group_control)
    }
    fn control_fixture_group_state_mut(&mut self) -> &mut FixtureGroupState {
        let active_fixture_group_control = self.active_fixture_group_control;

        self.active_scene_state_mut()
            .fixture_group_state_mut(active_fixture_group_control)
    }
    pub fn apply_event(&mut self, event: ControlEvent) {
        // dbg!(&event);
        match (&self.control_mode, event) {
            (_, ControlEvent::UpdateControlMode(mode)) => {
                self.control_mode = mode;
            }
            (_, ControlEvent::UpdateMasterDimmer(dimmer)) => {
                self.master_dimmer = dimmer;
            }
            (_, ControlEvent::UpdateDimmerEffectIntensity(intensity)) => {
                self.dimmer_effect_intensity = intensity;
            }
            (_, ControlEvent::UpdateColorEffectIntensity(intensity)) => {
                self.color_effect_intensity = intensity;
            }
            (_, ControlEvent::UpdateClockRate(rate)) => {
                let pressed_notes = self
                    .control_fixture_group_state()
                    .button_states
                    .pressed_notes();

                // If there are any buttons currently pressed, update the rate of those buttons, note the global rate
                if !pressed_notes.is_empty() {
                    self.control_fixture_group_state_mut()
                        .button_states
                        .update_pressed_button_rates(rate);
                } else {
                    self.control_fixture_group_state_mut().clock_rate = rate;
                }
            }
            (ControlMode::Normal, ControlEvent::SelectScene(scene_id)) => {
                self.active_scene_id = scene_id;
                self.active_fixture_group_control = None;
            }
            (ControlMode::Shift, ControlEvent::SelectScene(scene_id)) => {
                self.scene_fixture_group_button_states
                    .insert(scene_id, SceneState::default());
            }
            (ControlMode::Normal, ControlEvent::SelectFixtureGroupControl(group_id)) => {
                if Some(group_id) == self.active_fixture_group_control {
                    self.active_fixture_group_control = None;
                } else {
                    self.active_fixture_group_control = Some(group_id);
                }
            }
            (ControlMode::Shift, ControlEvent::SelectFixtureGroupControl(group_id)) => {
                self.active_scene_state_mut()
                    .fixture_groups
                    .insert(group_id, FixtureGroupState::default());
            }
            (_, ControlEvent::UpdateGroupDimmer(group_id, dimmer)) => {
                self.active_scene_state_mut()
                    .fixture_group_state_mut(Some(group_id))
                    .dimmer = dimmer;
            }
            (_, ControlEvent::UpdateButton(group, mapping, note_state, now)) => {
                self.control_fixture_group_state_mut()
                    .button_states
                    .update_button_state(&group, mapping, note_state, now);
            }
            (_, ControlEvent::TapTempo(now)) => {
                self.clock.apply_event(ClockEvent::Tap(now));
                dbg!(self.clock.bpm());
            }
        }
    }
    pub fn update_fixtures(&self, fixtures: &mut Vec<Fixture>) {
        let scene = self.active_scene_state();

        let clock_snapshot = self.clock.snapshot();
        let (base_values, fixture_group_values) = scene.fixture_group_values();

        let fixture_values = fixtures
            .iter()
            .map(|fixture| {
                let values = if let Some(group_id) = fixture.group_id {
                    fixture_group_values.get(&group_id).unwrap_or(&base_values)
                } else {
                    &base_values
                };

                let clock_snapshot = clock_snapshot.with_rate(values.clock_rate);

                let effect_dimmer = if fixture.dimmer_effects_enabled() {
                    values
                        .active_dimmer_effects
                        .iter()
                        .fold(1.0, |dimmer, (effect, rate)| {
                            dimmer
                                * effect::compress(
                                    effect.dimmer(&offsetted_for_fixture(
                                        effect.clock_offset.as_ref(),
                                        &clock_snapshot.with_rate(*rate),
                                        &fixture,
                                        &fixtures,
                                    )),
                                    self.dimmer_effect_intensity,
                                )
                        })
                } else {
                    1.0
                };

                let base_color = values.global_color().to_hsl();
                let secondary_color = values.secondary_color.map(Color::to_hsl);

                let color = if fixture.color_effects_enabled() {
                    effect::color_intensity(
                        base_color,
                        values.active_color_effects.iter().fold(
                            base_color,
                            |color, (effect, rate)| {
                                effect.color(
                                    color,
                                    secondary_color,
                                    &offsetted_for_fixture(
                                        effect.clock_offset.as_ref(),
                                        &clock_snapshot.with_rate(*rate),
                                        &fixture,
                                        &fixtures,
                                    ),
                                )
                            },
                        ),
                        self.color_effect_intensity,
                    )
                } else {
                    base_color
                };

                let pixel_range_set: Option<PixelRangeSet> = if fixture.pixel_effects_enabled() {
                    // TODO only using first active pixel effect
                    values
                        .active_pixel_effects
                        .iter()
                        .nth(0)
                        .map(|(effect, rate)| {
                            // reduce &&T to &T
                            let effect = *effect;
                            // Apply any active overrides
                            let overrides = &values.active_pixel_effect_overrides;
                            if !overrides.is_empty() {
                                let mut effect = effect.clone();
                                for effect_override in overrides.iter() {
                                    effect_override.apply(&mut effect);
                                }
                                (Cow::Owned(effect), rate)
                            } else {
                                (Cow::Borrowed(effect), rate)
                            }
                        })
                        .map(|(effect, rate)| {
                            effect.pixel_range_set(&offsetted_for_fixture(
                                effect.clock_offset.as_ref(),
                                &clock_snapshot.with_rate(*rate),
                                &fixture,
                                &fixtures,
                            ))
                        })
                } else {
                    None
                };

                let position = if fixture.position_effects_enabled() {
                    Some(
                        values
                            .active_position_effects
                            .iter()
                            .map(|(effect, rate)| {
                                effect.position(&offsetted_for_fixture(
                                    effect.clock_offset.as_ref(),
                                    &clock_snapshot.with_rate(*rate),
                                    &fixture,
                                    &fixtures,
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

                let dimmer = self.master_dimmer * group_dimmer * effect_dimmer;
                (dimmer, color, pixel_range_set, position)
            })
            .collect::<Vec<_>>();

        for (fixture, (dimmer, color, pixel_range, position)) in
            fixtures.iter_mut().zip(fixture_values)
        {
            fixture.set_dimmer(dimmer);
            fixture.set_color(color).unwrap();

            if fixture.profile.beam_count() > 1 {
                if let Some(pixel_range) = pixel_range {
                    fixture
                        .set_beam_dimmers(&pixel_range.pixel_dimmers(fixture.profile.beam_count()))
                } else {
                    // If there's no active pixel effect, reset pixels
                    fixture.set_all_beam_dimmers(1.0);
                }
            }

            if let Some(position) = position {
                fixture.set_position(position).unwrap();
            }
        }
    }
    fn meta_pad_events(&self) -> impl Iterator<Item = PadEvent<'_>> {
        let active_scene_button = self
            .midi_mapping
            .meta_buttons
            .values()
            .find(|button| button.on_action == MetaButtonAction::SelectScene(self.active_scene_id))
            .unwrap();

        let active_fixture_group_toggle_button =
            self.active_fixture_group_control
                .map(|control_fixture_group_id| {
                    self.midi_mapping
                        .meta_buttons
                        .values()
                        .find(|button| {
                            button.on_action
                                == MetaButtonAction::SelectFixtureGroupControl(
                                    control_fixture_group_id,
                                )
                        })
                        .unwrap()
                });

        let pressed_button_rate: Option<Rate> = self
            .control_fixture_group_state()
            .button_states
            .pressed_buttons()
            .values()
            .map(|(_, rate)| *rate)
            .max();

        let clock_rate = self.control_fixture_group_state().clock_rate;
        let active_clock_rate_button = self
            .midi_mapping
            .meta_buttons
            .values()
            .find(|button| {
                button.on_action
                    == MetaButtonAction::UpdateClockRate(pressed_button_rate.unwrap_or(clock_rate))
            })
            .unwrap();

        vec![
            Some(active_scene_button),
            Some(active_clock_rate_button),
            active_fixture_group_toggle_button,
        ]
        .into_iter()
        .flatten()
        .map(PadEvent::new_on)
    }
    pub fn pad_events(&self) -> impl Iterator<Item = PadEvent<'_>> {
        self.control_fixture_group_state()
            .button_states
            .iter_info()
            .map(PadEvent::from)
            .chain(self.meta_pad_events())
    }
}
