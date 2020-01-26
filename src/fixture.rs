use serde::Deserialize;

#[derive(Debug, Clone, Copy, Deserialize)]
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

#[derive(Debug, Clone, Deserialize)]
struct FixtureProfile {
    slug: String,
    label: String,
    channel_count: usize,
    channels: Vec<FixtureProfileChannel>,
}
