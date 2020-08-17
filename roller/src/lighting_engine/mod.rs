use derive_more::Constructor;
use rustc_hash::FxHashMap;
use std::time::Instant;

use roller_protocol::{
    clock::{Clock, ClockEvent, Rate},
    color::Color,
    control::{NoteState,InputEvent},
    effect::{ColorEffect, DimmerEffect, PixelEffect, PositionEffect},
    fixture::{FixtureGroupId},
    position::BasePosition,
};

use crate::{
    control::{
        button::{ButtonGroup, ButtonMapping, ButtonRef, MetaButtonAction},
        control_mapping::ControlMapping,
        fader::fader_mapping_control_event
    },
    utils::FxIndexMap,
};

mod button_states;
pub mod render;

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
    pub dimmer_effect_intensity: Option<f64>,
    pub color_effect_intensity: Option<f64>,
    pub clock_rate: Rate,
    pub global_color: Option<Color>,
    pub secondary_color: Option<Color>,
    pub base_position: Option<BasePosition>,
    pub active_dimmer_effects: FxIndexMap<&'a DimmerEffect, Rate>,
    pub active_color_effects: FxIndexMap<&'a ColorEffect, Rate>,
    pub active_pixel_effects: FxIndexMap<&'a PixelEffect, Rate>,
    pub active_position_effects: FxIndexMap<&'a PositionEffect, Rate>,
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
        if self.dimmer_effect_intensity == None {
            self.dimmer_effect_intensity = other.dimmer_effect_intensity;
        }
        if self.color_effect_intensity == None {
            self.color_effect_intensity = other.color_effect_intensity;
        }
        self.active_dimmer_effects
            .extend(other.active_dimmer_effects.iter());
        self.active_color_effects
            .extend(other.active_color_effects.iter());
        self.active_pixel_effects
            .extend(other.active_pixel_effects.iter());
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
    pub fn dimmer_effect_intensity(&self) -> f64 {
        self.dimmer_effect_intensity.unwrap_or(0.5)
    }
    pub fn color_effect_intensity(&self) -> f64 {
        self.color_effect_intensity.unwrap_or(1.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ControlMode {
    Shift,
    Normal,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ControlEvent<'a> {
    UpdateMasterDimmer(f64),
    UpdateGroupDimmer(FixtureGroupId, f64),
    UpdateDimmerEffectIntensity(f64),
    UpdateColorEffectIntensity(f64),
    UpdateClockRate(Rate),
    SelectScene(SceneId),
    SelectFixtureGroupControl(FixtureGroupId),
    UpdateButton(&'a ButtonGroup, &'a ButtonMapping, NoteState, Instant),
    TapTempo(Instant),
    UpdateControlMode(ControlMode),
}

pub struct EngineState<'a> {
    pub control_mapping: &'a ControlMapping,
    pub clock: Clock,
    pub master_dimmer: f64,
    pub control_mode: ControlMode,
    pub active_scene_id: SceneId,
    pub active_fixture_group_control: Option<FixtureGroupId>,
    pub scene_fixture_group_button_states: FxHashMap<SceneId, SceneState>,
}
impl<'a> EngineState<'a> {
    pub fn new(control_mapping: &'a ControlMapping) -> EngineState<'a> {
        EngineState {
            control_mapping,
            clock: Clock::new(128.0),
            master_dimmer: 1.0,
            control_mode: ControlMode::Normal,
            active_scene_id: SceneId::new(1),
            active_fixture_group_control: None,
            scene_fixture_group_button_states: FxHashMap::default(),
        }
    }
    pub fn active_scene_state(&self) -> &SceneState {
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
    pub fn apply_input_event(&mut self, event: InputEvent) {
        let now = Instant::now();

        let control_event = match event {
            InputEvent::FaderUpdated(fader_id, value) => self
                .control_mapping
                .faders
                .get(&fader_id)
                .map(|fader| fader_mapping_control_event(fader, value)),
            InputEvent::ButtonPressed(location, coordinate) => self
                .control_mapping
                .find_button(location, coordinate)
                .and_then(|button_ref| button_ref.into_control_event(NoteState::On, now)),
            InputEvent::ButtonReleased(location, coordinate) => self
                .control_mapping
                .find_button(location, coordinate)
                .and_then(|button_ref| button_ref.into_control_event(NoteState::Off, now)),
        };

        if let Some(control_event) = control_event {
            self.apply_event(control_event);
        }
    }
    fn apply_event(&mut self, event: ControlEvent) {
        // dbg!(&event);
        match (&self.control_mode, event) {
            (_, ControlEvent::UpdateControlMode(mode)) => {
                self.control_mode = mode;
            }
            (_, ControlEvent::UpdateMasterDimmer(dimmer)) => {
                self.master_dimmer = dimmer;
            }
            (_, ControlEvent::UpdateDimmerEffectIntensity(intensity)) => {
                self.active_scene_state_mut().dimmer_effect_intensity = intensity;
            }
            (_, ControlEvent::UpdateColorEffectIntensity(intensity)) => {
                self.active_scene_state_mut().color_effect_intensity = intensity;
            }
            (_, ControlEvent::UpdateClockRate(rate)) => {
                let pressed_coords = self
                    .control_fixture_group_state()
                    .button_states
                    .pressed_coords();

                // If there are any buttons currently pressed, update the rate of those buttons, note the global rate
                if !pressed_coords.is_empty() {
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
                    .update_button_state(&group, mapping.clone(), note_state, now);
            }
            (_, ControlEvent::TapTempo(now)) => {
                self.clock.apply_event(ClockEvent::Tap(now));
                dbg!(self.clock.bpm());
            }
        }
    }
    fn meta_input_events(&self) -> impl Iterator<Item = InputEvent> + '_ {
        let active_scene_button = self
            .control_mapping
            .meta_buttons
            .values()
            .find(|button| button.on_action == MetaButtonAction::SelectScene(self.active_scene_id))
            .unwrap();

        let active_fixture_group_toggle_button =
            self.active_fixture_group_control
                .map(|control_fixture_group_id| {
                    self.control_mapping
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
            .control_mapping
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
        .map(ButtonRef::from)
        .map(|button_ref| {
            InputEvent::ButtonPressed(button_ref.location(), *button_ref.coordinate())
        })
    }
    pub fn input_events(&self) -> impl Iterator<Item = InputEvent> + '_ {
        self.control_fixture_group_state()
            .button_states
            .iter_info()
            .map(|(group_info, button_info)| {
                let button_ref = ButtonRef::Standard(group_info.group, button_info.button);

                match button_info.note_state {
                    NoteState::On => {
                        InputEvent::ButtonPressed(button_ref.location(), *button_ref.coordinate())
                    }
                    NoteState::Off => {
                        InputEvent::ButtonReleased(button_ref.location(), *button_ref.coordinate())
                    }
                }
            })
            .chain(self.meta_input_events())
    }
}
