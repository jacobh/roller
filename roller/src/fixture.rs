use async_std::{prelude::*, sync::Arc};
use rustc_hash::FxHashMap;
use serde::Deserialize;

use crate::project::GroupId;

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
    parameters: FxHashMap<FixtureParameter, FixtureProfileChannel>,
}
impl FixtureProfile {
    pub async fn load(
        path: impl AsRef<async_std::path::Path>,
    ) -> Result<FixtureProfile, async_std::io::Error> {
        let fixture_profile_contents = async_std::fs::read(path).await?;
        let profile_data: FixtureProfileData = toml::from_slice(&fixture_profile_contents)?;

        let parameters = profile_data
            .channels
            .iter()
            .map(|channel| (channel.parameter, channel.clone()))
            .collect();

        // Ensure channel count is correct
        assert_eq!(profile_data.channel_count, profile_data.channels.len());
        Ok(FixtureProfile {
            parameters,
            data: profile_data,
        })
    }
    pub fn is_dimmable(&self) -> bool {
        self.parameters.get(&FixtureParameter::Dimmer).is_some()
    }
    pub fn is_colorable(&self) -> bool {
        [
            FixtureParameter::Red,
            FixtureParameter::Green,
            FixtureParameter::Blue,
        ]
        .into_iter()
        .map(|parameter| self.parameters.get(parameter))
        .all(|channel| channel.is_some())
    }
    pub fn is_positionable(&self) -> bool {
        [FixtureParameter::Tilt, FixtureParameter::Pan]
            .into_iter()
            .map(|parameter| self.parameters.get(parameter))
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
    pub group_id: Option<GroupId>,

    dimmer: f64, // 0.0 - 1.0
    color: Option<palette::LinSrgb<f64>>,
    position: Option<(f64, f64)>, // -1.0 - +1.0
}
impl Fixture {
    pub fn new(
        profile: Arc<FixtureProfile>,
        universe: usize,
        start_channel: usize,
        group_id: Option<GroupId>,
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
        let parameters = &self.profile.parameters;
        let mut dmx: Vec<u8> = (0..self.profile.data.channel_count).map(|_| 0).collect();

        if self.profile.is_dimmable() {
            let channel = &parameters[&FixtureParameter::Dimmer];

            dmx[channel.channel_index()] = channel.encode_value(self.dimmer);
        }

        if let Some(color) = self.color {
            let (mut red, mut green, mut blue) = color.into_components();

            // If light doesn't have dimmer control, scale the color values instead
            if !self.profile.is_dimmable() {
                red = red * self.dimmer;
                green = green * self.dimmer;
                blue = blue * self.dimmer;
            }

            let red_channel = &parameters[&FixtureParameter::Red];
            let green_channel = &parameters[&FixtureParameter::Green];
            let blue_channel = &parameters[&FixtureParameter::Blue];

            dmx[red_channel.channel_index()] = red_channel.encode_value(red);
            dmx[green_channel.channel_index()] = green_channel.encode_value(green);
            dmx[blue_channel.channel_index()] = blue_channel.encode_value(blue);
        }

        if let Some(position) = self.position {
            let tilt_channel = &parameters[&FixtureParameter::Tilt];
            let pan_channel = &parameters[&FixtureParameter::Pan];

            dmx[tilt_channel.channel_index()] = tilt_channel.encode_value((position.1 + 1.0) / 2.0);
            dmx[pan_channel.channel_index()] = pan_channel.encode_value((position.0 + 1.0) / 2.0);
        }

        dmx
    }
    pub fn absolute_dmx(&self) -> Vec<Option<u8>> {
        (0..(self.start_channel - 1))
            .map(|_| None)
            .chain(self.relative_dmx().into_iter().map(Some))
            .collect()
    }
}
