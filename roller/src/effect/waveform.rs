use std::f64::consts::PI;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub enum Waveform {
    SawUp,
    SawDown,
    TriangleDown,
    SineUp,
    SineDown,
    HalfSineUp,
    HalfSineDown,
    ShortSquarePulse,
    HalfRootUp,
    HalfRootDown,
    OnePointFiveRootUp,
    OnePointFiveRootDown,
    On,
    Off,
}
impl Waveform {
    pub fn apply(self, x: f64) -> f64 {
        match self {
            Waveform::SawUp => saw_up(x),
            Waveform::SawDown => saw_down(x),
            Waveform::TriangleDown => triangle_down(x),
            Waveform::SineUp => sine_up(x),
            Waveform::SineDown => sine_down(x),
            Waveform::HalfSineUp => half_sine_up(x),
            Waveform::HalfSineDown => half_sine_down(x),
            Waveform::ShortSquarePulse => short_square_pulse(x),
            Waveform::HalfRootUp => root(x, 0.5),
            Waveform::HalfRootDown => invert(root(x, 0.5)),
            Waveform::OnePointFiveRootUp => root(x, 1.5),
            Waveform::OnePointFiveRootDown => invert(root(x, 1.5)),
            Waveform::On => 1.0,
            Waveform::Off => 0.0,
        }
    }
}

// Waveforms for `x` in the range 0.0 - 1.0
fn saw_up(x: f64) -> f64 {
    x
}

fn saw_down(x: f64) -> f64 {
    1.0 - x
}

fn triangle_down(x: f64) -> f64 {
    if x > 0.5 {
        (x - 0.5) * 2.0
    } else {
        1.0 - (x * 2.0)
    }
}

/// 0.0 = 0.0
/// 0.5 = 1.0
/// 1.0 = 0.0
fn sine_up(x: f64) -> f64 {
    (f64::sin(PI * 2.0 * x - 1.5) / 2.0) + 0.5
}

/// 0.0 = 1.0
/// 0.5 = 0.0
/// 1.0 = 1.0
fn sine_down(x: f64) -> f64 {
    (f64::sin(PI * 2.0 * x + 1.5) / 2.0) + 0.5
}

fn half_sine_up(x: f64) -> f64 {
    (f64::sin(((PI * 2.0 * x) / 2.0) - 1.5) / 2.0) + 0.5
}

fn half_sine_down(x: f64) -> f64 {
    (f64::sin(((PI * 2.0 * x) / 2.0) + 1.5) / 2.0) + 0.5
}

fn short_square_pulse(x: f64) -> f64 {
    if x < 0.2 {
        1.0
    } else {
        f64::max(0.5 - (x / 1.2), 0.0)
    }
}

fn root(x: f64, root: f64) -> f64 {
    f64::powf(x, 1.0 / root)
}

fn invert(x: f64) -> f64 {
    1.0 - x
}
