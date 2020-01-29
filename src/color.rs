use palette::Hsl;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Color {
    White,
    Yellow,
    DeepOrange,
    Red,
    Violet,
    DarkBlue,
    Teal,
    Green,
}
impl Color {
    pub fn to_hsl(self) -> Hsl<palette::encoding::srgb::Srgb, f64> {
        match self {
            Color::White => Hsl::new(0.0, 0.0, 1.0),
            Color::Yellow => Hsl::new(42.0, 1.0, 0.5),
            Color::DeepOrange => Hsl::new(32.0, 1.0, 0.5),
            Color::Red => Hsl::new(0.0, 1.0, 0.5),
            Color::Violet => Hsl::new(270.0, 1.0, 0.5),
            Color::DarkBlue => Hsl::new(240.0, 1.0, 0.5),
            Color::Teal => Hsl::new(180.0, 1.0, 0.5),
            Color::Green => Hsl::new(120.0, 1.0, 0.5),
        }
    }
    pub fn to_rgb(self) -> palette::LinSrgb<f64> {
        palette::LinSrgb::from(self.to_hsl())
    }
}
impl Into<palette::LinSrgb<f64>> for Color {
    fn into(self) -> palette::LinSrgb<f64> {
        self.to_rgb()
    }
}
impl Into<Hsl<palette::encoding::srgb::Srgb, f64>> for Color {
    fn into(self) -> Hsl<palette::encoding::srgb::Srgb, f64> {
        self.to_hsl()
    }
}
