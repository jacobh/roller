pub type DimmerEffect = Box<dyn Fn(f64) -> f64>;

pub fn saw_up(progress: f64) -> f64 {
    progress
}

pub fn saw_down(progress: f64) -> f64 {
    1.0 - progress
}

pub fn triangle_down(progress: f64) -> f64 {
    if progress > 0.5 {
        (progress - 0.5) * 2.0
    } else {
        1.0 - (progress * 2.0)
    }
}

pub fn sine(progress: f64) -> f64 {
    (f64::sin(std::f64::consts::PI * 2.0 * progress) / 2.0) + 0.5
}
