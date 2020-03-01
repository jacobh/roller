use async_std::{prelude::*, sync::Arc};
use derive_more::{Constructor, From, Into};
use rustc_hash::FxHashMap;
use serde::Deserialize;

use crate::project::FixtureGroupId;
use crate::utils::FxIndexMap;

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Constructor, Deserialize, From, Into,
)]
pub struct BeamId(usize);

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

    beams: FxIndexMap<Option<BeamId>, FixtureBeamProfile>,
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
                        beam.dimmer_channel = Some(channel.clone());
                    }
                    FixtureParameter::Red => {
                        beam.red_channel = Some(channel.clone());
                    }
                    FixtureParameter::Green => {
                        beam.green_channel = Some(channel.clone());
                    }
                    FixtureParameter::Blue => {
                        beam.blue_channel = Some(channel.clone());
                    }
                    _ => {}
                }

                beams
            });

        beams.sort_keys();

        // Ensure channel count is correct
        assert_eq!(profile_data.channel_count, profile_data.channels.len());

        // assert have at least 1 beam
        assert!(beams.len() > 0);
        Ok(FixtureProfile {
            beams,
            pan_channel: parameters.get(&FixtureParameter::Pan).cloned(),
            tilt_channel: parameters.get(&FixtureParameter::Tilt).cloned(),
            data: profile_data,
        })
    }
    pub fn beam_count(&self) -> usize {
        let beam_count = self.beams.keys().filter(|id| id.is_some()).count();

        usize::max(beam_count, 1)
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
) -> Result<FxHashMap<String, Arc<FixtureProfile>>, async_std::io::Error> {
    let mut profile_paths = async_std::fs::read_dir("./fixture_profiles").await?;

    let mut fixture_profiles = FxHashMap::default();
    while let Some(entry) = profile_paths.next().await {
        let path = entry?.path();

        let fixture_profile = FixtureProfile::load(path).await?;
        fixture_profiles.insert(fixture_profile.data.slug.clone(), Arc::new(fixture_profile));
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
}

#[derive(Debug, Clone, PartialEq)]
pub struct Fixture {
    pub profile: Arc<FixtureProfile>,
    universe: usize,
    start_channel: usize,
    pub group_id: Option<FixtureGroupId>,

    beams: FxIndexMap<Option<BeamId>, FixtureBeam>,
    position: Option<(f64, f64)>, // -1.0 - +1.0
}
impl Fixture {
    pub fn new(
        profile: Arc<FixtureProfile>,
        universe: usize,
        start_channel: usize,
        group_id: Option<FixtureGroupId>,
    ) -> Fixture {
        let beams = profile
            .beams
            .iter()
            .map(|(beam_id, profile)| (*beam_id, FixtureBeam::new(profile.clone())))
            .collect();

        Fixture {
            profile,
            universe,
            start_channel,
            group_id,
            beams,
            position: None,
        }
    }
    fn global_beam_mut(&mut self) -> Option<&mut FixtureBeam> {
        self.beams.get_mut(&None)
    }
    fn beams_mut(&mut self) -> impl Iterator<Item = (BeamId, &mut FixtureBeam)> {
        self.beams
            .iter_mut()
            .filter_map(|(beam_id, beam)| beam_id.map(|beam_id| (beam_id, beam)))
    }
    pub fn set_dimmer(&mut self, dimmer: f64) {
        if let Some(beam) = self.global_beam_mut() {
            beam.dimmer = dimmer;
        } else {
            for (_, beam) in self.beams_mut() {
                beam.dimmer = dimmer;
            }
        }
    }
    pub fn set_beam_dimmers(&mut self, dimmers: &[f64]) {
        for ((_, beam), dimmer) in self.beams_mut().zip(dimmers) {
            beam.dimmer = *dimmer;
        }
    }
    pub fn set_color(
        &mut self,
        color: impl Into<palette::LinSrgb<f64>>,
    ) -> Result<(), &'static str> {
        let color = color.into();

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
    pub fn set_position(&mut self, position: (f64, f64)) -> Result<(), &'static str> {
        if self.profile.is_positionable() {
            self.position = Some(position);
            Ok(())
        } else {
            Err("Unable to set position. profile does not support it")
        }
    }
    pub fn relative_dmx(&self) -> Vec<u8> {
        let mut dmx: Vec<u8> = vec![0; self.profile.data.channel_count];

        for beam in self.beams.values() {
            if let Some(channel) = &beam.profile.dimmer_channel {
                dmx[channel.channel_index()] = channel.encode_value(beam.dimmer);
            }

            if let (Some(color), Some((red_channel, green_channel, blue_channel))) =
                (beam.color, beam.profile.color_channels())
            {
                let (mut red, mut green, mut blue) = color.into_components();

                // If light doesn't have dimmer control, scale the color values instead
                if !beam.profile.is_dimmable() {
                    red *= beam.dimmer;
                    green *= beam.dimmer;
                    blue *= beam.dimmer;
                }

                dmx[red_channel.channel_index()] = red_channel.encode_value(red);
                dmx[green_channel.channel_index()] = green_channel.encode_value(green);
                dmx[blue_channel.channel_index()] = blue_channel.encode_value(blue);
            }
        }

        if let (Some(position), Some(tilt_channel), Some(pan_channel)) = (
            self.position,
            &self.profile.tilt_channel,
            &self.profile.pan_channel,
        ) {
            dmx[tilt_channel.channel_index()] = tilt_channel.encode_value((position.1 + 1.0) / 2.0);
            dmx[pan_channel.channel_index()] = pan_channel.encode_value((position.0 + 1.0) / 2.0);
        }

        dmx
    }
    pub fn write_dmx(&self, dmx: &mut [u8]) {
        for (i, channel) in self.relative_dmx().into_iter().enumerate() {
            dmx[i + self.start_channel - 1] = channel
        }
    }
}

pub fn fold_fixture_dmx_data<'a>(fixtures: impl IntoIterator<Item = &'a Fixture>) -> [u8; 512] {
    let mut dmx_data = [0; 512];

    for fixture in fixtures {
        fixture.write_dmx(&mut dmx_data);
    }

    dmx_data
}
