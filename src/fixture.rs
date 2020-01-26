use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
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
struct FixtureProfile {
    slug: String,
    label: String,
    channel_count: usize,
    channels: Vec<FixtureProfileChannel>,
}
impl FixtureProfile {
    fn is_colorable(&self) -> bool {
        self.channels.contains(&FixtureProfileChannel::Red)
            && self.channels.contains(&FixtureProfileChannel::Green)
            && self.channels.contains(&FixtureProfileChannel::Blue)
    }
    fn is_positionable(&self) -> bool {
        self.channels.contains(&FixtureProfileChannel::Tilt)
            && self.channels.contains(&FixtureProfileChannel::Pan)
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
struct Fixture {
    profile: FixtureProfile,
    universe: usize,
    start_channel: usize,

    dimmer: f64,  // 0.0 - 1.0
    color: Option<(u8, u8, u8)>,
    position: Option<(f64, f64)>, // -1.0 - +1.0
}
impl Fixture {
    fn set_dimmer(&mut self, dimmer: f64) {
        self.dimmer = dimmer;
    }
    fn set_color(&mut self, color: (u8, u8, u8)) -> Result<(), &'static str> {
        if self.profile.is_colorable() {
            self.color = Some(color);
            Ok(())
        } else {
            Err("Unable to set color. profile does not support it")
        }
    }
    fn set_position(&mut self, position: (f64, f64)) -> Result<(), &'static str> {
        if self.profile.is_positionable() {
            self.position = Some(position);
            Ok(())
        } else {
            Err("Unable to set position. profile does not support it")
        }
    }
}
