#![recursion_limit = "1024"]

use wasm_bindgen::prelude::*;

mod app;
mod button_grid;
mod ui;
mod utils;
mod page;

#[wasm_bindgen]
pub fn run_app() -> Result<(), JsValue> {
    yew::start_app::<app::App>();

    Ok(())
}
