use async_std::prelude::*;
use rustc_hash::FxHashMap;
use serde::Deserialize;

use crate::utils::FxIndexMap;

use roller_protocol::fixture::*;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct FixtureProfileData {
    slug: String,
    label: String,
    channel_count: usize,
    channels: Vec<FixtureProfileChannel>,
    supported_effects: Vec<FixtureEffectType>,
}

pub async fn load_fixture_profile(
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
        slug: profile_data.slug,
        label: profile_data.label,
        channel_count: profile_data.channel_count,
        supported_effects: profile_data.supported_effects,

        beams,
        dimmer_channel,
        pan_channel: parameters.get(&FixtureParameter::Pan).cloned(),
        tilt_channel: parameters.get(&FixtureParameter::Tilt).cloned(),
    })
}

pub async fn load_fixture_profiles(
) -> Result<FxHashMap<String, FixtureProfile>, async_std::io::Error> {
    let mut profile_paths = async_std::fs::read_dir("./fixture_profiles").await?;

    let mut fixture_profiles = FxHashMap::default();
    while let Some(entry) = profile_paths.next().await {
        let path = entry?.path();

        let fixture_profile = load_fixture_profile(path).await?;
        fixture_profiles.insert(fixture_profile.slug.clone(), fixture_profile);
    }

    Ok(fixture_profiles)
}
