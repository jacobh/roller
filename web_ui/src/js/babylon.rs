use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    pub type Engine;

    #[wasm_bindgen(constructor, js_namespace = BABYLON)]
    pub fn new(canvas_element: web_sys::HtmlCanvasElement, x: bool) -> Engine;

    // static get Version(): string;
    #[wasm_bindgen(static_method_of = Engine, getter, js_namespace = BABYLON, js_name = "Version")]
    pub fn version() -> String;

    pub type Scene;

    #[wasm_bindgen(constructor, js_namespace = BABYLON)]
    pub fn new(engine: &Engine) -> Scene;
}
