use async_std::{prelude::*, sync::Arc};
use serde::Deserialize;
use rustc_hash::FxHashMap;

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
    pub async fn load(
        path: impl AsRef<async_std::path::Path>,
    ) -> Result<FixtureProfile, async_std::io::Error> {
        let fixture_profile_contents = async_std::fs::read(path).await?;
        let profile: FixtureProfile = toml::from_slice(&fixture_profile_contents)?;

        // Ensure channel count is correct
        assert_eq!(profile.channel_count, profile.channels.len());
        Ok(profile)
    }
    pub fn is_dimmable(&self) -> bool {
        self.channels.contains(&FixtureProfileChannel::Dimmer)
    }
    pub fn is_colorable(&self) -> bool {
        self.channels.contains(&FixtureProfileChannel::Red)
            && self.channels.contains(&FixtureProfileChannel::Green)
            && self.channels.contains(&FixtureProfileChannel::Blue)
    }
    pub fn is_positionable(&self) -> bool {
        self.channels.contains(&FixtureProfileChannel::Tilt)
            && self.channels.contains(&FixtureProfileChannel::Pan)
    }
    fn channel_index(&self, channel: FixtureProfileChannel) -> Option<usize> {
        self.channels.iter().position(|x| *x == channel)
    }
}

pub async fn load_fixture_profiles(
) -> Result<FxHashMap<String, Arc<FixtureProfile>>, async_std::io::Error> {
    let mut profile_paths = async_std::fs::read_dir("./fixture_profiles").await?;

    let mut fixture_profiles = FxHashMap::default();
    while let Some(entry) = profile_paths.next().await {
        let path = entry?.path();

        let fixture_profile = FixtureProfile::load(path).await?;
        fixture_profiles.insert(fixture_profile.slug.clone(), Arc::new(fixture_profile));
    }

    Ok(fixture_profiles)
}

#[derive(Debug, Clone, PartialEq)]
pub struct Fixture {
    pub profile: Arc<FixtureProfile>,
    universe: usize,
    start_channel: usize,
    pub group_id: Option<usize>,

    dimmer: f64, // 0.0 - 1.0
    color: Option<palette::LinSrgb<f64>>,
    position: Option<(f64, f64)>, // -1.0 - +1.0
}
impl Fixture {
    pub fn new(
        profile: Arc<FixtureProfile>,
        universe: usize,
        start_channel: usize,
        group_id: Option<usize>,
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
        let mut dmx: Vec<u8> = (0..self.profile.channel_count).map(|_| 0).collect();

        if self.profile.is_dimmable() {
            dmx[self
                .profile
                .channel_index(FixtureProfileChannel::Dimmer)
                .unwrap()] = (255 as f64 * self.dimmer) as u8;
        }

        if let Some(color) = self.color {
            let (mut red, mut green, mut blue) = color.into_components();

            // If light doesn't have dimmer control, scale the color values instead
            if !self.profile.is_dimmable() {
                red = red * self.dimmer;
                green = green * self.dimmer;
                blue = blue * self.dimmer;
            }

            dmx[self
                .profile
                .channel_index(FixtureProfileChannel::Red)
                .unwrap()] = (255.0 * red) as u8;
            dmx[self
                .profile
                .channel_index(FixtureProfileChannel::Green)
                .unwrap()] = (255.0 * green) as u8;
            dmx[self
                .profile
                .channel_index(FixtureProfileChannel::Blue)
                .unwrap()] = (255.0 * blue) as u8;
        }

        if let Some(position) = self.position {
            dmx[self
                .profile
                .channel_index(FixtureProfileChannel::Tilt)
                .unwrap()] = (255.0 * ((position.1 + 1.0) / 2.0)) as u8;
            dmx[self
                .profile
                .channel_index(FixtureProfileChannel::Pan)
                .unwrap()] = (255.0 * ((position.0 + 1.0) / 2.0)) as u8;
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
