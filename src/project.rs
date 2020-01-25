use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
struct ProjectFixture {
    start_channel: usize,
    fixture_profile_slug: String,
}

#[derive(Debug, Clone, Deserialize)]
struct ProjectUniverse {
    universe_id: usize,
    fixtures: Vec<ProjectFixture>
}

#[derive(Debug, Clone, Deserialize)]
struct Project {
    label: String,
    universes: Vec<ProjectUniverse>
}
