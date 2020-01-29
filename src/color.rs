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
    Green
}
impl Color {
    pub fn to_hsl(self) -> Hsl {
        match self {
            Color::White => Hsl::new(0.0, 0.0, 1.0),
            Color::Yellow => Hsl::new(42.0, 1.0, 1.0),
            Color::DeepOrange => Hsl::new(32.0, 1.0, 1.0),
            Color::Red => Hsl::new(0.0, 1.0, 1.0),
            Color::Violet => Hsl::new(270.0, 1.0, 1.0),
            Color::DarkBlue => Hsl::new(240.0, 1.0, 1.0),
            Color::Teal => Hsl::new(180.0, 1.0, 1.0),
            Color::Green => Hsl::new(120.0, 1.0, 1.0),
        }
    }
    pub fn to_rgb(self) -> palette::LinSrgb {
        palette::LinSrgb::from(self.to_hsl())
    }
}
