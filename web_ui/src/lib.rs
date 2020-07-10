#![recursion_limit = "1024"]

use wasm_bindgen::prelude::*;

mod app;
mod button_grid;
mod pages;
mod ui;
mod utils;
mod yewtil;

pub mod pure {
    pub use crate::yewtil::pure::{Pure, PureComponent};
}

#[wasm_bindgen]
pub fn run_app() -> Result<(), JsValue> {
    yew::start_app::<app::App>();

    Ok(())
}
