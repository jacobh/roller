use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[derive(Debug)]
    pub type Engine;

    #[wasm_bindgen(constructor, js_namespace = BABYLON)]
    pub fn new(canvas_element: &web_sys::HtmlCanvasElement, x: bool) -> Engine;

    // static get Version(): string;
    #[wasm_bindgen(static_method_of = Engine, getter, js_name = "Version", js_namespace = BABYLON)]
    pub fn version() -> String;

    #[derive(Debug)]
    pub type Scene;

    #[wasm_bindgen(constructor, js_namespace = BABYLON)]
    pub fn new(engine: &Engine) -> Scene;
}
