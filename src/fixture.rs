use async_std::{prelude::*, sync::Arc};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum FixtureProfileChannel {
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
pub struct FixtureProfile {
    slug: String,
    label: String,
    channel_count: usize,
    channels: Vec<FixtureProfileChannel>,
}
impl FixtureProfile {
    fn is_dimmable(&self) -> bool {
        self.channels.contains(&FixtureProfileChannel::Dimmer)
    }
    fn is_colorable(&self) -> bool {
        self.channels.contains(&FixtureProfileChannel::Red)
            && self.channels.contains(&FixtureProfileChannel::Green)
            && self.channels.contains(&FixtureProfileChannel::Blue)
    }
    fn is_positionable(&self) -> bool {
        self.channels.contains(&FixtureProfileChannel::Tilt)
            && self.channels.contains(&FixtureProfileChannel::Pan)
    }
    fn channel_index(&self, channel: FixtureProfileChannel) -> Option<usize> {
        self.channels.iter().position(|x| *x == channel)
    }
}

pub async fn load_fixture_profiles(
) -> Result<HashMap<String, Arc<FixtureProfile>>, async_std::io::Error> {
    let mut profile_paths = async_std::fs::read_dir("./fixture_profiles").await?;

    let mut fixture_profiles = HashMap::new();
    while let Some(entry) = profile_paths.next().await {
        let path = entry?.path();
        let fixture_profile_contents = async_std::fs::read(path).await?;

        let fixture_profile: FixtureProfile = toml::from_slice(&fixture_profile_contents)?;
        fixture_profiles.insert(fixture_profile.slug.clone(), Arc::new(fixture_profile));
    }

    Ok(fixture_profiles)
}

#[derive(Debug, Clone, PartialEq)]
pub struct Fixture {
    profile: Arc<FixtureProfile>,
    universe: usize,
    start_channel: usize,

    dimmer: f64, // 0.0 - 1.0
    color: Option<(u8, u8, u8)>,
    position: Option<(f64, f64)>, // -1.0 - +1.0
}
impl Fixture {
    pub fn new(profile: Arc<FixtureProfile>, universe: usize, start_channel: usize) -> Fixture {
        Fixture {
            profile,
            universe,
            start_channel,
            dimmer: 1.0,
            color: None,
            position: None,
        }
    }
    pub fn set_dimmer(&mut self, dimmer: f64) {
        self.dimmer = dimmer;
    }
    pub fn set_color(&mut self, color: (u8, u8, u8)) -> Result<(), &'static str> {
        if self.profile.is_colorable() {
            self.color = Some(color);
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
        let mut dmx: Vec<u8> = (0..self.profile.channel_count).map(|_| 0).collect();

        if self.profile.is_dimmable() {
            dmx[self
                .profile
                .channel_index(FixtureProfileChannel::Dimmer)
                .unwrap()] = (255 as f64 * self.dimmer) as u8;
        }

        if let Some(color) = self.color {
            let (mut red, mut green, mut blue) = color;

            // If light doesn't have dimmer control, scale the color values instead
            if !self.profile.is_dimmable() {
                red = (red as f64 * self.dimmer) as u8;
                green = (green as f64 * self.dimmer) as u8;
                blue = (blue as f64 * self.dimmer) as u8;
            }

            dmx[self
                .profile
                .channel_index(FixtureProfileChannel::Red)
                .unwrap()] = red;
            dmx[self
                .profile
                .channel_index(FixtureProfileChannel::Green)
                .unwrap()] = green;
            dmx[self
                .profile
                .channel_index(FixtureProfileChannel::Blue)
                .unwrap()] = blue;
        }

        // TODO positions

        dmx
    }
    pub fn absolute_dmx(&self) -> Vec<Option<u8>> {
        (0..(self.start_channel - 1))
            .map(|_| None)
            .chain(self.relative_dmx().into_iter().map(Some))
            .collect()
    }
}
