use derive_more::{Constructor, From, Into};
use rustc_hash::FxHashSet;
use serde::Deserialize;

use crate::fixture::{Fixture, FixtureEffectType};

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Constructor, Deserialize, From, Into,
)]
pub struct FixtureGroupId(usize);

#[derive(Debug, Clone, Deserialize)]
struct ProjectFixture {
    start_channel: usize,
    group_id: Option<FixtureGroupId>,
    location: Option<FixtureLocation>,
    #[serde(rename = "fixture_profile")]
    fixture_profile_slug: String,
    #[serde(default = "FixtureEffectType::all")]
    enabled_effects: FxHashSet<FixtureEffectType>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Constructor, Deserialize)]
pub struct FixtureLocation {
    pub x: isize,
    pub y: isize,
}

#[derive(Debug, Clone, Deserialize)]
struct ProjectUniverse {
    universe_id: usize,
    fixtures: Vec<ProjectFixture>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Project {
    label: String,
    pub midi_controller: Option<String>,
    pub midi_clock: Option<String>,
    universes: Vec<ProjectUniverse>,
}
impl Project {
    pub async fn load(
        path: impl AsRef<async_std::path::Path>,
    ) -> Result<Project, async_std::io::Error> {
        let config_file_contents = async_std::fs::read(path).await?;

        let project: Project = toml::from_slice(&config_file_contents)?;
        Ok(project)
    }
    pub async fn fixtures(&self) -> Result<Vec<Fixture>, async_std::io::Error> {
        let fixture_profiles = crate::fixture::load_fixture_profiles().await?;

        let fixtures = self
            .universes
            .iter()
            .cloned()
            .flat_map(|universe| {
                let universe_id = universe.universe_id;

                universe
                    .fixtures
                    .into_iter()
                    .map(|project_fixture| {
                        let profile = fixture_profiles
                            .get(&project_fixture.fixture_profile_slug)
                            .unwrap()
                            .clone();

                        Fixture::new(
                            profile,
                            universe_id,
                            project_fixture.start_channel,
                            project_fixture.group_id,
                            project_fixture.location,
                            project_fixture.enabled_effects,
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        // TODO validate fixture addresses don't overlap

        Ok(fixtures)
    }
}
