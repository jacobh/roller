use serde::Deserialize;

#[derive(Debug, Clone, Copy, Deserialize)]
enum FixtureProfileChannel {
    Red,
    Green,
    Blue,
    CoolWhite,
    Pan,
    Tilt,
    Unused
}

#[derive(Debug, Clone, Deserialize)]
struct FixtureProfile {
    slug: String,
    label: String,
    channels: Vec<FixtureProfileChannel>
}
