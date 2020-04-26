#![recursion_limit = "256"]

mod app;
mod button;
mod button_grid;

use wasm_bindgen::prelude::*;

#[derive(Debug, Clone)]
pub enum ButtonGridLocation {
    Main,
    MetaRight,
    MetaBottom
}
impl ButtonGridLocation {
    fn css_name(&self) -> &'static str {
        match self {
            ButtonGridLocation::Main => "main",
            ButtonGridLocation::MetaRight => "meta-right",
            ButtonGridLocation::MetaBottom => "meta-bottom",
        }
    }
}

#[derive(Debug, Clone)]
pub enum ButtonState {
    Active,
    Inactive,
    Deactivated,
    Unused
}
impl ButtonState {
    fn css_class(&self) -> &'static str {
        match self {
            ButtonState::Active => "button--active",
            ButtonState::Inactive => "button--inactive",
            ButtonState::Deactivated => "button--deactivated",
            ButtonState::Unused => "button--unused",
        }
    }
}
impl Default for ButtonState {
    fn default() -> ButtonState {
        ButtonState::Unused
    }
}

#[wasm_bindgen]
pub fn run_app() -> Result<(), JsValue> {
    yew::start_app::<app::App>();

    Ok(())
}
