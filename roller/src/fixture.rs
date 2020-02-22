use async_std::{prelude::*, sync::Arc};
use rustc_hash::FxHashMap;
use serde::Deserialize;

use crate::project::FixtureGroupId;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureProfile {
    data: FixtureProfileData,

    dimmer_channel: Option<FixtureProfileChannel>,
    red_channel: Option<FixtureProfileChannel>,
    green_channel: Option<FixtureProfileChannel>,
    blue_channel: Option<FixtureProfileChannel>,
    cool_white_channel: Option<FixtureProfileChannel>,
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

        // Ensure channel count is correct
        assert_eq!(profile_data.channel_count, profile_data.channels.len());
        Ok(FixtureProfile {
            dimmer_channel: parameters.get(&FixtureParameter::Dimmer).cloned(),
            red_channel: parameters.get(&FixtureParameter::Red).cloned(),
            green_channel: parameters.get(&FixtureParameter::Green).cloned(),
            blue_channel: parameters.get(&FixtureParameter::Blue).cloned(),
            cool_white_channel: parameters.get(&FixtureParameter::CoolWhite).cloned(),
            pan_channel: parameters.get(&FixtureParameter::Pan).cloned(),
            tilt_channel: parameters.get(&FixtureParameter::Tilt).cloned(),

            data: profile_data,
        })
    }
    pub fn is_dimmable(&self) -> bool {
        self.dimmer_channel.is_some()
    }
    pub fn is_colorable(&self) -> bool {
        [&self.red_channel, &self.green_channel, &self.blue_channel]
            .iter()
            .all(|channel| channel.is_some())
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
pub struct Fixture {
    pub profile: Arc<FixtureProfile>,
    universe: usize,
    start_channel: usize,
    pub group_id: Option<FixtureGroupId>,

    dimmer: f64, // 0.0 - 1.0
    color: Option<palette::LinSrgb<f64>>,
    position: Option<(f64, f64)>, // -1.0 - +1.0
}
impl Fixture {
    pub fn new(
        profile: Arc<FixtureProfile>,
        universe: usize,
        start_channel: usize,
        group_id: Option<FixtureGroupId>,
    ) -> Fixture {
        Fixture {
            profile,
            universe,
            start_channel,
            group_id,
            dimmer: 1.0,
            color: None,
            position: None,
        }
    }
    pub fn set_dimmer(&mut self, dimmer: f64) {
        self.dimmer = dimmer;
    }
    pub fn set_color(
        &mut self,
        color: impl Into<palette::LinSrgb<f64>>,
    ) -> Result<(), &'static str> {
        if self.profile.is_colorable() {
            self.color = Some(color.into());
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

        if let Some(channel) = &self.profile.dimmer_channel {
            dmx[channel.channel_index()] = channel.encode_value(self.dimmer);
        }

        if let (Some(color), Some(red_channel), Some(green_channel), Some(blue_channel)) = (
            self.color,
            &self.profile.red_channel,
            &self.profile.green_channel,
            &self.profile.blue_channel,
        ) {
            let (mut red, mut green, mut blue) = color.into_components();

            // If light doesn't have dimmer control, scale the color values instead
            if !self.profile.is_dimmable() {
                red *= self.dimmer;
                green *= self.dimmer;
                blue *= self.dimmer;
            }

            dmx[red_channel.channel_index()] = red_channel.encode_value(red);
            dmx[green_channel.channel_index()] = green_channel.encode_value(green);
            dmx[blue_channel.channel_index()] = blue_channel.encode_value(blue);
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
