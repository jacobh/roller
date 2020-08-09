use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[derive(Debug)]
    pub type Vector3;

    #[wasm_bindgen(constructor, js_namespace = BABYLON)]
    pub fn new(x: f64, y: f64, z: f64) -> Vector3;

    #[wasm_bindgen(method, getter, js_namespace = BABYLON)]
    fn x(this: &Vector3) -> f64;

    #[wasm_bindgen(method, getter, js_namespace = BABYLON)]
    fn y(this: &Vector3) -> f64;

    #[wasm_bindgen(method, getter, js_namespace = BABYLON)]
    fn z(this: &Vector3) -> f64;

    #[derive(Debug)]
    pub type Vector4;

    #[wasm_bindgen(constructor, js_namespace = BABYLON)]
    pub fn new(w: f64, x: f64, y: f64, z: f64) -> Vector4;

    #[wasm_bindgen(method, getter, js_namespace = BABYLON)]
    fn w(this: &Vector4) -> f64;

    #[wasm_bindgen(method, getter, js_namespace = BABYLON)]
    fn x(this: &Vector4) -> f64;

    #[wasm_bindgen(method, getter, js_namespace = BABYLON)]
    fn y(this: &Vector4) -> f64;

    #[wasm_bindgen(method, getter, js_namespace = BABYLON)]
    fn z(this: &Vector4) -> f64;

    #[derive(Debug)]
    pub type EngineOptions;

    #[derive(Debug)]
    pub type Engine;

    #[wasm_bindgen(constructor, js_namespace = BABYLON)]
    pub fn new(
        canvas_element: &web_sys::HtmlCanvasElement,
        antialias: Option<bool>,
        options: Option<EngineOptions>,
        adapt_to_device_ratio: Option<bool>,
    ) -> Engine;

    // static get Version(): string;
    #[wasm_bindgen(static_method_of = Engine, getter, js_name = "Version", js_namespace = BABYLON)]
    pub fn version() -> String;

    #[derive(Debug)]
    pub type Scene;

    #[wasm_bindgen(constructor, js_namespace = BABYLON)]
    pub fn new(engine: &Engine) -> Scene;

    #[derive(Debug)]
    pub type ArcRotateCamera;

    #[wasm_bindgen(constructor, js_namespace = BABYLON)]
    pub fn new(
        name: String,
        alpha: f64,
        beta: f64,
        radius: f64,
        target: Vector3,
        scene: &Scene,
        set_active_on_scene_if_none_active: Option<bool>,
    ) -> ArcRotateCamera;

    #[wasm_bindgen(method, js_name="attachControl", js_namespace = BABYLON)]
    pub fn attach_control(
        this: &ArcRotateCamera,
        element: &web_sys::HtmlElement,
        no_prevent_default: Option<bool>,
        use_ctrl_for_panning: Option<bool>,
        panning_mouse_button: Option<usize>,
    );

    #[derive(Debug)]
    pub type HemisphericLight;

    #[wasm_bindgen(constructor, js_namespace = BABYLON)]
    pub fn new(name: String, direction: Vector3, scene: &Scene) -> HemisphericLight;

    #[derive(Debug)]
    pub type PointLight;

    #[wasm_bindgen(constructor, js_namespace = BABYLON)]
    pub fn new(name: String, position: Vector3, scene: &Scene) -> PointLight;

    #[derive(Debug)]
    pub type Mesh;

    #[derive(Debug)]
    pub type MeshBuilder;

    #[wasm_bindgen(static_method_of = MeshBuilder, js_name="CreateSphere", js_namespace = BABYLON)]
    pub fn create_sphere(name: String, options: CreateSphereOptions, scene: Option<&Scene>)
        -> Mesh;
}

#[wasm_bindgen]
#[derive(Debug, Default)]
pub struct CreateSphereOptions {
    pub arc: Option<f64>,
    pub backUVs: Option<bool>, // TODO should be `Vector4`
    pub diameter: Option<f64>,
    pub diameterX: Option<f64>,
    pub diameterY: Option<f64>,
    pub diameterZ: Option<f64>,
    pub frontUVs: Option<bool>, // TODO should be `Vector4`
    pub segments: Option<f64>,
    pub sideOrientation: Option<f64>,
    pub slice: Option<f64>,
    pub updatable: Option<bool>,
}
