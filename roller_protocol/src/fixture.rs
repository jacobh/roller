use derive_more::{Constructor, From, Into};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

use crate::position::{degrees_to_percent, Position};

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, From, Into, Serialize, Deserialize,
)]
pub struct FixtureId(uuid::Uuid);
impl FixtureId {
    fn new() -> FixtureId {
        FixtureId(uuid::Uuid::new_v4())
    }
}

#[derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Constructor,
    From,
    Into,
    Serialize,
    Deserialize,
)]
pub struct FixtureGroupId(usize);

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Constructor, Serialize, Deserialize)]
pub struct FixtureLocation {
    pub x: isize,
    pub y: isize,
}

#[derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Constructor,
    Serialize,
    Deserialize,
    From,
    Into,
)]
pub struct BeamId(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "snake_case")]
pub enum FixtureEffectType {
    Color,
    Dimmer,
    Pixel,
    Position,
}
impl FixtureEffectType {
    pub fn all() -> Vec<FixtureEffectType> {
        vec![
            FixtureEffectType::Color,
            FixtureEffectType::Dimmer,
            FixtureEffectType::Pixel,
            FixtureEffectType::Position,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "snake_case")]
pub enum FixtureParameter {
    Dimmer,
    Red,
    Green,
    Blue,
    CoolWhite,
    Pan,
    Tilt,
    Unused,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FixtureProfileChannel {
    pub parameter: FixtureParameter,
    pub channel: usize,
    pub beam: Option<BeamId>,
    #[serde(default = "FixtureProfileChannel::default_min_value")]
    pub min_value: u8,
    #[serde(default = "FixtureProfileChannel::default_max_value")]
    pub max_value: u8,
}
impl FixtureProfileChannel {
    const fn default_min_value() -> u8 {
        0
    }
    const fn default_max_value() -> u8 {
        255
    }
    pub fn channel_index(&self) -> usize {
        self.channel - 1
    }
    // value in range 0.0 - 1.0
    pub fn encode_value(&self, value: f64) -> u8 {
        let range = self.max_value - self.min_value;

        self.min_value + (range as f64 * value) as u8
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct FixtureBeamProfile {
    pub dimmer_channel: Option<FixtureProfileChannel>,
    pub red_channel: Option<FixtureProfileChannel>,
    pub green_channel: Option<FixtureProfileChannel>,
    pub blue_channel: Option<FixtureProfileChannel>,
    pub cool_white_channel: Option<FixtureProfileChannel>,
}
impl FixtureBeamProfile {
    pub fn is_dimmable(&self) -> bool {
        self.dimmer_channel.is_some()
    }
    pub fn is_colorable(&self) -> bool {
        [&self.red_channel, &self.green_channel, &self.blue_channel]
            .iter()
            .all(|channel| channel.is_some())
    }
    pub fn color_channels(
        &self,
    ) -> Option<(
        &FixtureProfileChannel,
        &FixtureProfileChannel,
        &FixtureProfileChannel,
    )> {
        match (
            self.red_channel.as_ref(),
            self.green_channel.as_ref(),
            self.blue_channel.as_ref(),
        ) {
            (Some(red_channel), Some(blue_channel), Some(green_channel)) => {
                Some((red_channel, blue_channel, green_channel))
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FixtureProfile {
    pub slug: String,
    pub label: String,
    pub channel_count: usize,
    pub supported_effects: Vec<FixtureEffectType>,

    pub beams: Vec<FixtureBeamProfile>,
    pub dimmer_channel: Option<FixtureProfileChannel>,
    pub pan_channel: Option<FixtureProfileChannel>,
    pub tilt_channel: Option<FixtureProfileChannel>,
}
impl FixtureProfile {
    pub fn beam_count(&self) -> usize {
        self.beams.len()
    }
    pub fn is_dimmable(&self) -> bool {
        self.beams.iter().any(FixtureBeamProfile::is_dimmable)
    }
    pub fn is_colorable(&self) -> bool {
        self.beams.iter().any(FixtureBeamProfile::is_colorable)
    }
    pub fn is_positionable(&self) -> bool {
        [&self.pan_channel, &self.tilt_channel]
            .iter()
            .all(|channel| channel.is_some())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FixtureBeamState {
    pub dimmer: f64,
    pub color: Option<(f64, f64, f64)>,
}
impl FixtureBeamState {
    pub fn new() -> FixtureBeamState {
        FixtureBeamState {
            dimmer: 1.0,
            color: None,
        }
    }
    pub fn is_white(&self) -> bool {
        match self.color {
            Some(color) => color == (1.0, 1.0, 1.0),
            None => false,
        }
    }
    pub fn color(&self) -> Option<palette::LinSrgb<f64>> {
        self.color.map(palette::LinSrgb::from_components)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FixtureParams {
    pub id: FixtureId,
    pub profile: FixtureProfile,
    pub universe: usize,
    pub start_channel: usize,
    pub group_id: Option<FixtureGroupId>,
    pub location: Option<FixtureLocation>,
    pub enabled_effects: Vec<FixtureEffectType>,
}
impl FixtureParams {
    fn enabled_effects(&self) -> impl Iterator<Item = FixtureEffectType> + '_ {
        self.profile
            .supported_effects
            .clone()
            .into_iter()
            .filter(move |effect| self.enabled_effects.contains(&effect))
    }
    pub fn dimmer_effects_enabled(&self) -> bool {
        self.enabled_effects()
            .any(|x| x == FixtureEffectType::Dimmer)
    }
    pub fn color_effects_enabled(&self) -> bool {
        self.enabled_effects()
            .any(|x| x == FixtureEffectType::Color)
    }
    pub fn pixel_effects_enabled(&self) -> bool {
        self.enabled_effects()
            .any(|x| x == FixtureEffectType::Pixel)
    }
    pub fn position_effects_enabled(&self) -> bool {
        self.enabled_effects()
            .any(|x| x == FixtureEffectType::Position)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FixtureState {
    pub beams: Vec<FixtureBeamState>,
    pub dimmer: f64,
    pub position: Option<Position>, // degrees from home position
}
impl FixtureState {
    pub fn new(profile: &FixtureProfile) -> FixtureState {
        let beams = profile
            .beams
            .iter()
            .map(|_| FixtureBeamState::new())
            .collect();

        FixtureState {
            beams,
            dimmer: 1.0,
            position: None,
        }
    }
    pub fn set_dimmer(&mut self, dimmer: f64) {
        self.dimmer = dimmer;
    }
    pub fn set_beam_dimmers(&mut self, dimmers: &[f64]) {
        for (beam, dimmer) in self.beams.iter_mut().zip(dimmers) {
            beam.dimmer = *dimmer;
        }
    }
    pub fn set_all_beam_dimmers(&mut self, dimmer: f64) {
        for beam in self.beams.iter_mut() {
            beam.dimmer = dimmer;
        }
    }
    pub fn set_color(&mut self, color: impl Into<palette::LinSrgb<f64>>) {
        let color = color.into();

        // Apply a subtle perceptual brightness scale to colors.
        // Luma values have been eyeballed and might differ on a per-fixture basis in reality
        const R_LUMA: f64 = 1.2;
        const G_LUMA: f64 = 1.7;
        const B_LUMA: f64 = 1.0;
        let (r, g, b) = color.into_components();
        let is_white = r == 1.0 && g == 1.0 && b == 1.0;

        // Special casing white here for it to remain full brightness
        // TODO some sort of enum like `BrightnessMode {Full, Perceuptual}`
        let scale = if !is_white {
            let luminance = r * R_LUMA + g * G_LUMA + b * B_LUMA;
            1.0 / luminance * B_LUMA
        } else {
            1.0
        };

        let color = palette::LinSrgb::<f64>::new(r * scale, g * scale, b * scale);

        for beam in self.beams.iter_mut() {
            beam.color = Some(color.into_components());
        }
    }
    pub fn set_position(&mut self, position: Position) {
        self.position = Some(position);
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Fixture {
    pub params: FixtureParams,
    pub state: FixtureState,
}
impl Fixture {
    pub fn new(
        profile: FixtureProfile,
        universe: usize,
        start_channel: usize,
        group_id: Option<FixtureGroupId>,
        location: Option<FixtureLocation>,
        enabled_effects: Vec<FixtureEffectType>,
    ) -> Fixture {
        Fixture {
            state: FixtureState::new(&profile),
            params: FixtureParams {
                id: FixtureId::new(),
                profile,
                universe,
                start_channel,
                group_id,
                location,
                enabled_effects,
            },
        }
    }
    pub fn id(&self) -> &FixtureId {
        &self.params.id
    }
    pub fn relative_dmx(&self) -> Vec<u8> {
        let mut dmx: Vec<u8> = vec![0; self.params.profile.channel_count];

        if let Some(dimmer_channel) = &self.params.profile.dimmer_channel {
            dmx[dimmer_channel.channel_index()] = dimmer_channel.encode_value(self.state.dimmer)
        }

        let beam_profiles = self
            .params
            .profile
            .beams
            .iter()
            .zip(self.state.beams.iter());

        for (beam_profile, beam_state) in beam_profiles {
            // If fixture has a global dimmer, use that, otherwise dim on a per beam basis
            let beam_dimmer = if self.params.profile.dimmer_channel.is_some() {
                beam_state.dimmer
            } else {
                beam_state.dimmer * self.state.dimmer
            };

            if let Some(channel) = &beam_profile.dimmer_channel {
                dmx[channel.channel_index()] = channel.encode_value(beam_dimmer);
            }

            if let (Some(color), Some((red_channel, green_channel, blue_channel))) =
                (beam_state.color, beam_profile.color_channels())
            {
                let (mut red, mut green, mut blue) = color;

                // Custom calibrated warmer white
                if beam_state.is_white() {
                    red = 1.0;
                    green = 0.906;
                    blue = 0.49;
                }

                // If light doesn't have dimmer control, scale the color values instead
                if !beam_profile.is_dimmable() {
                    red *= beam_dimmer;
                    green *= beam_dimmer;
                    blue *= beam_dimmer;
                }

                dmx[red_channel.channel_index()] = red_channel.encode_value(red);
                dmx[green_channel.channel_index()] = green_channel.encode_value(green);
                dmx[blue_channel.channel_index()] = blue_channel.encode_value(blue);
            }

            if let Some(white_channel) = beam_profile.cool_white_channel.as_ref() {
                if beam_state.is_white() {
                    let white_value = if beam_profile.is_dimmable() {
                        1.0
                    } else {
                        beam_dimmer
                    };

                    dmx[white_channel.channel_index()] = white_channel.encode_value(white_value);
                }
            }
        }

        if let (Some(position), Some(tilt_channel), Some(pan_channel)) = (
            self.state.position,
            &self.params.profile.tilt_channel,
            &self.params.profile.pan_channel,
        ) {
            // TODO move to fixture profile
            const PAN_RANGE: f64 = 540.0;
            const TILT_RANGE: f64 = 180.0;

            let pan_value = degrees_to_percent(position.pan(), PAN_RANGE);
            let tilt_value = degrees_to_percent(position.tilt(), TILT_RANGE);

            dmx[pan_channel.channel_index()] = pan_channel.encode_value(pan_value);
            dmx[tilt_channel.channel_index()] = tilt_channel.encode_value(tilt_value);
        }

        dmx
    }
    pub fn write_dmx(&self, dmx: &mut [u8]) {
        for (i, channel) in self.relative_dmx().into_iter().enumerate() {
            dmx[i + self.params.start_channel - 1] = channel
        }
    }
}

pub fn fold_fixture_dmx_data<'a>(
    fixtures: impl IntoIterator<Item = &'a Fixture>,
) -> FxHashMap<usize, [u8; 512]> {
    let mut universe_dmx_data = FxHashMap::default();
    universe_dmx_data.reserve(1);

    for fixture in fixtures {
        let dmx_data = universe_dmx_data
            .entry(fixture.params.universe)
            .or_insert_with(|| [0u8; 512]);

        fixture.write_dmx(dmx_data);
    }

    universe_dmx_data
}
