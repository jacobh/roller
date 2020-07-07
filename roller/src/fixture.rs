use async_std::prelude::*;
use derive_more::{Constructor, From, Into};
use rustc_hash::{FxHashMap, FxHashSet};
use serde::Deserialize;

use roller_protocol::position::{degrees_to_percent, Position};

use crate::{
    project::{FixtureGroupId, FixtureLocation},
    utils::FxIndexMap,
};

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Constructor, Deserialize, From, Into,
)]
pub struct BeamId(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Hash)]
#[serde(rename_all = "snake_case")]
pub enum FixtureEffectType {
    Color,
    Dimmer,
    Pixel,
    Position,
}
impl FixtureEffectType {
    pub fn all() -> FxHashSet<FixtureEffectType> {
        vec![
            FixtureEffectType::Color,
            FixtureEffectType::Dimmer,
            FixtureEffectType::Pixel,
            FixtureEffectType::Position,
        ]
        .into_iter()
        .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Hash)]
#[serde(rename_all = "snake_case")]
enum FixtureParameter {
    Dimmer,
    Red,
    Green,
    Blue,
    CoolWhite,
    Pan,
    Tilt,
    Unused,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct FixtureProfileChannel {
    parameter: FixtureParameter,
    channel: usize,
    beam: Option<BeamId>,
    #[serde(default = "FixtureProfileChannel::default_min_value")]
    min_value: u8,
    #[serde(default = "FixtureProfileChannel::default_max_value")]
    max_value: u8,
}
impl FixtureProfileChannel {
    const fn default_min_value() -> u8 {
        0
    }
    const fn default_max_value() -> u8 {
        255
    }
    fn channel_index(&self) -> usize {
        self.channel - 1
    }
    // value in range 0.0 - 1.0
    fn encode_value(&self, value: f64) -> u8 {
        let range = self.max_value - self.min_value;

        self.min_value + (range as f64 * value) as u8
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct FixtureProfileData {
    slug: String,
    label: String,
    channel_count: usize,
    channels: Vec<FixtureProfileChannel>,
    supported_effects: FxHashSet<FixtureEffectType>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FixtureBeamProfile {
    dimmer_channel: Option<FixtureProfileChannel>,
    red_channel: Option<FixtureProfileChannel>,
    green_channel: Option<FixtureProfileChannel>,
    blue_channel: Option<FixtureProfileChannel>,
    cool_white_channel: Option<FixtureProfileChannel>,
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
    fn color_channels(
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureProfile {
    data: FixtureProfileData,

    beams: FxIndexMap<BeamId, FixtureBeamProfile>,
    dimmer_channel: Option<FixtureProfileChannel>,
    pan_channel: Option<FixtureProfileChannel>,
    tilt_channel: Option<FixtureProfileChannel>,
}
impl FixtureProfile {
    pub async fn load(
        path: impl AsRef<async_std::path::Path>,
    ) -> Result<FixtureProfile, async_std::io::Error> {
        let fixture_profile_contents = async_std::fs::read(path).await?;
        let profile_data: FixtureProfileData = toml::from_slice(&fixture_profile_contents)?;

        let parameters: FxHashMap<_, _> = profile_data
            .channels
            .iter()
            .map(|channel| (channel.parameter, channel.clone()))
            .collect();

        let mut beams: FxIndexMap<Option<BeamId>, FixtureBeamProfile> = profile_data
            .channels
            .iter()
            .fold(FxIndexMap::default(), |mut beams, channel| {
                let mut beam = beams.entry(channel.beam).or_default();

                match channel.parameter {
                    FixtureParameter::Dimmer => {
                        assert!(beam.dimmer_channel.is_none());
                        beam.dimmer_channel = Some(channel.clone());
                    }
                    FixtureParameter::Red => {
                        assert!(beam.red_channel.is_none());
                        beam.red_channel = Some(channel.clone());
                    }
                    FixtureParameter::Green => {
                        assert!(beam.green_channel.is_none());
                        beam.green_channel = Some(channel.clone());
                    }
                    FixtureParameter::Blue => {
                        assert!(beam.blue_channel.is_none());
                        beam.blue_channel = Some(channel.clone());
                    }
                    FixtureParameter::CoolWhite => {
                        assert!(beam.cool_white_channel.is_none());
                        beam.cool_white_channel = Some(channel.clone());
                    }
                    _ => {}
                }

                beams
            });
        beams.sort_keys();

        // Pluck out the default dimmer channel
        let dimmer_channel = beams
            .get(&None)
            .and_then(|beam| beam.dimmer_channel.clone());
        beams
            .entry(None)
            .and_modify(|beam| beam.dimmer_channel = None);

        // If beams have been configured, use those, otherwise, give the default beam an ID
        let (default_beam, beams): (Vec<_>, Vec<_>) =
            beams.into_iter().partition(|(id, _)| id.is_none());

        let beams: FxIndexMap<_, _> = if beams.len() > 0 {
            beams
                .into_iter()
                .map(|(id, beam)| (id.unwrap(), beam))
                .collect()
        } else {
            default_beam
                .into_iter()
                .map(|(_, beam)| (BeamId::new(0), beam))
                .collect()
        };

        // Ensure channel count is correct
        assert_eq!(profile_data.channel_count, profile_data.channels.len());

        // assert have at least 1 beam
        assert!(beams.len() > 0);

        Ok(FixtureProfile {
            beams,
            dimmer_channel,
            pan_channel: parameters.get(&FixtureParameter::Pan).cloned(),
            tilt_channel: parameters.get(&FixtureParameter::Tilt).cloned(),
            data: profile_data,
        })
    }
    pub fn beam_count(&self) -> usize {
        self.beams.len()
    }
    pub fn is_dimmable(&self) -> bool {
        self.beams.values().any(FixtureBeamProfile::is_dimmable)
    }
    pub fn is_colorable(&self) -> bool {
        self.beams.values().any(FixtureBeamProfile::is_colorable)
    }
    pub fn is_positionable(&self) -> bool {
        [&self.pan_channel, &self.tilt_channel]
            .iter()
            .all(|channel| channel.is_some())
    }
}

pub async fn load_fixture_profiles(
) -> Result<FxHashMap<String, FixtureProfile>, async_std::io::Error> {
    let mut profile_paths = async_std::fs::read_dir("./fixture_profiles").await?;

    let mut fixture_profiles = FxHashMap::default();
    while let Some(entry) = profile_paths.next().await {
        let path = entry?.path();

        let fixture_profile = FixtureProfile::load(path).await?;
        fixture_profiles.insert(fixture_profile.data.slug.clone(), fixture_profile);
    }

    Ok(fixture_profiles)
}

#[derive(Debug, Clone, PartialEq)]
pub struct FixtureBeam {
    profile: FixtureBeamProfile,
    dimmer: f64,
    color: Option<palette::LinSrgb<f64>>,
}
impl FixtureBeam {
    fn new(profile: FixtureBeamProfile) -> FixtureBeam {
        FixtureBeam {
            profile,
            dimmer: 1.0,
            color: None,
        }
    }
    fn is_white(&self) -> bool {
        match self.color {
            Some(color) => color.into_components() == (1.0, 1.0, 1.0),
            None => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Fixture {
    pub profile: FixtureProfile,
    universe: usize,
    start_channel: usize,
    pub group_id: Option<FixtureGroupId>,
    pub location: Option<FixtureLocation>,
    enabled_effects: FxHashSet<FixtureEffectType>,

    beams: FxIndexMap<BeamId, FixtureBeam>,
    dimmer: f64,
    position: Option<Position>, // degrees from home position
}
impl Fixture {
    pub fn new(
        profile: FixtureProfile,
        universe: usize,
        start_channel: usize,
        group_id: Option<FixtureGroupId>,
        location: Option<FixtureLocation>,
        enabled_effects: FxHashSet<FixtureEffectType>,
    ) -> Fixture {
        let beams = profile
            .beams
            .clone()
            .into_iter()
            .map(|(beam_id, profile)| (beam_id, FixtureBeam::new(profile)))
            .collect();

        Fixture {
            profile,
            universe,
            start_channel,
            group_id,
            location,
            enabled_effects,
            beams,
            dimmer: 1.0,
            position: None,
        }
    }
    fn enabled_effects(&self) -> impl Iterator<Item = &FixtureEffectType> {
        self.profile
            .data
            .supported_effects
            .intersection(&self.enabled_effects)
    }
    pub fn dimmer_effects_enabled(&self) -> bool {
        self.enabled_effects()
            .any(|x| x == &FixtureEffectType::Dimmer)
    }
    pub fn color_effects_enabled(&self) -> bool {
        self.enabled_effects()
            .any(|x| x == &FixtureEffectType::Color)
    }
    pub fn pixel_effects_enabled(&self) -> bool {
        self.enabled_effects()
            .any(|x| x == &FixtureEffectType::Pixel)
    }
    pub fn position_effects_enabled(&self) -> bool {
        self.enabled_effects()
            .any(|x| x == &FixtureEffectType::Position)
    }
    pub fn set_dimmer(&mut self, dimmer: f64) {
        self.dimmer = dimmer;
    }
    pub fn set_beam_dimmers(&mut self, dimmers: &[f64]) {
        for (beam, dimmer) in self.beams.values_mut().zip(dimmers) {
            beam.dimmer = *dimmer;
        }
    }
    pub fn set_all_beam_dimmers(&mut self, dimmer: f64) {
        for beam in self.beams.values_mut() {
            beam.dimmer = dimmer;
        }
    }
    pub fn set_color(
        &mut self,
        color: impl Into<palette::LinSrgb<f64>>,
    ) -> Result<(), &'static str> {
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

        if self.profile.is_colorable() {
            for beam in self.beams.values_mut() {
                if beam.profile.is_colorable() {
                    beam.color = Some(color);
                }
            }
            Ok(())
        } else {
            Err("Unable to set color. profile does not support it")
        }
    }
    pub fn set_position(&mut self, position: Position) -> Result<(), &'static str> {
        if self.profile.is_positionable() {
            self.position = Some(position);
            Ok(())
        } else {
            Err("Unable to set position. profile does not support it")
        }
    }
    pub fn relative_dmx(&self) -> Vec<u8> {
        let mut dmx: Vec<u8> = vec![0; self.profile.data.channel_count];

        if let Some(dimmer_channel) = &self.profile.dimmer_channel {
            dmx[dimmer_channel.channel_index()] = dimmer_channel.encode_value(self.dimmer)
        }

        for beam in self.beams.values() {
            // If fixture has a global dimmer, use that, otherwise dim on a per beam basis
            let beam_dimmer = if self.profile.dimmer_channel.is_some() {
                beam.dimmer
            } else {
                beam.dimmer * self.dimmer
            };

            if let Some(channel) = &beam.profile.dimmer_channel {
                dmx[channel.channel_index()] = channel.encode_value(beam_dimmer);
            }

            if let (Some(color), Some((red_channel, green_channel, blue_channel))) =
                (beam.color, beam.profile.color_channels())
            {
                let (mut red, mut green, mut blue) = color.into_components();

                // Custom calibrated warmer white
                if beam.is_white() {
                    red = 1.0;
                    green = 0.906;
                    blue = 0.49;
                }

                // If light doesn't have dimmer control, scale the color values instead
                if !beam.profile.is_dimmable() {
                    red *= beam_dimmer;
                    green *= beam_dimmer;
                    blue *= beam_dimmer;
                }

                dmx[red_channel.channel_index()] = red_channel.encode_value(red);
                dmx[green_channel.channel_index()] = green_channel.encode_value(green);
                dmx[blue_channel.channel_index()] = blue_channel.encode_value(blue);
            }

            if let Some(white_channel) = beam.profile.cool_white_channel.as_ref() {
                if beam.is_white() {
                    let white_value = if beam.profile.is_dimmable() {
                        1.0
                    } else {
                        beam_dimmer
                    };

                    dmx[white_channel.channel_index()] = white_channel.encode_value(white_value);
                }
            }
        }

        if let (Some(position), Some(tilt_channel), Some(pan_channel)) = (
            self.position,
            &self.profile.tilt_channel,
            &self.profile.pan_channel,
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
            dmx[i + self.start_channel - 1] = channel
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
            .entry(fixture.universe)
            .or_insert_with(|| [0u8; 512]);

        fixture.write_dmx(dmx_data);
    }

    universe_dmx_data
}
