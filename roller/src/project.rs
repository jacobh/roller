use derive_more::{Constructor, From, Into};
use serde::Deserialize;

use crate::fixture::Fixture;

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Constructor, Deserialize, From, Into,
)]
pub struct FixtureGroupId(usize);

#[derive(Debug, Clone, Deserialize)]
struct ProjectFixture {
    start_channel: usize,
    group_id: Option<FixtureGroupId>,
    #[serde(rename = "fixture_profile")]
    fixture_profile_slug: String,
}

#[derive(Debug, Clone, Deserialize)]
struct ProjectUniverse {
    universe_id: usize,
    fixtures: Vec<ProjectFixture>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Project {
    label: String,
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
            .flat_map(|universe| {
                universe
                    .fixtures
                    .iter()
                    .map(|project_fixture| {
                        let profile = fixture_profiles
                            .get(&project_fixture.fixture_profile_slug)
                            .unwrap()
                            .clone();
                        Fixture::new(
                            profile,
                            universe.universe_id,
                            project_fixture.start_channel,
                            project_fixture.group_id,
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        // TODO validate fixture addresses don't overlap

        Ok(fixtures)
    }
}
