#![recursion_limit = "1024"]

use wasm_bindgen::prelude::*;

mod app;
mod button_grid;
mod pages;
mod ui;
mod utils;
mod yewtil;

#[wasm_bindgen]
pub fn run_app() -> Result<(), JsValue> {
    yew::start_app::<app::App>();

    Ok(())
}
