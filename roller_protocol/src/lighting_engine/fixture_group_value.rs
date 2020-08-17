use crate::{
    clock::Rate,
    color::Color,
    effect::{ColorEffect, DimmerEffect, PixelEffect, PositionEffect},
    position::BasePosition,
    utils::FxIndexMap,
};

#[derive(Default)]
pub struct FixtureGroupValue {
    pub dimmer: f64,
    pub dimmer_effect_intensity: Option<f64>,
    pub color_effect_intensity: Option<f64>,
    pub clock_rate: Rate,
    pub global_color: Option<Color>,
    pub secondary_color: Option<Color>,
    pub base_position: Option<BasePosition>,
    pub active_dimmer_effects: FxIndexMap<DimmerEffect, Rate>,
    pub active_color_effects: FxIndexMap<ColorEffect, Rate>,
    pub active_pixel_effects: FxIndexMap<PixelEffect, Rate>,
    pub active_position_effects: FxIndexMap<PositionEffect, Rate>,
}
impl FixtureGroupValue {
    pub fn merge(mut self, other: &FixtureGroupValue) -> FixtureGroupValue {
        self.clock_rate = self.clock_rate * other.clock_rate;
        if self.global_color == None {
            self.global_color = other.global_color;
        }
        if self.secondary_color == None {
            self.secondary_color = other.secondary_color;
        }
        if self.base_position == None {
            self.base_position = other.base_position;
        }
        if self.dimmer_effect_intensity == None {
            self.dimmer_effect_intensity = other.dimmer_effect_intensity;
        }
        if self.color_effect_intensity == None {
            self.color_effect_intensity = other.color_effect_intensity;
        }
        self.active_dimmer_effects
            .extend(other.active_dimmer_effects.clone().into_iter());
        self.active_color_effects
            .extend(other.active_color_effects.clone().into_iter());
        self.active_pixel_effects
            .extend(other.active_pixel_effects.clone().into_iter());
        self.active_position_effects
            .extend(other.active_position_effects.clone().into_iter());

        self
    }
    pub fn global_color(&self) -> Color {
        self.global_color.unwrap_or(Color::Violet)
    }
    pub fn base_position(&self) -> BasePosition {
        self.base_position.unwrap_or_default()
    }
    pub fn dimmer_effect_intensity(&self) -> f64 {
        self.dimmer_effect_intensity.unwrap_or(0.5)
    }
    pub fn color_effect_intensity(&self) -> f64 {
        self.color_effect_intensity.unwrap_or(1.0)
    }
}
