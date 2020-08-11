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

    #[wasm_bindgen(method, js_namespace = BABYLON, js_name="runRenderLoop")]
    pub fn run_render_loop(this: &Engine, render: &Closure<dyn FnMut()>);

    // static get Version(): string;
    #[wasm_bindgen(static_method_of = Engine, getter, js_name = "Version", js_namespace = BABYLON)]
    pub fn version() -> String;

    #[derive(Debug, Clone)]
    pub type Scene;

    #[wasm_bindgen(constructor, js_namespace = BABYLON)]
    pub fn new(engine: &Engine) -> Scene;

    #[wasm_bindgen(method, js_namespace = BABYLON)]
    pub fn render(this: &Scene, update_cameras: Option<bool>, ignore_animation: Option<bool>);

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
    pub type Light;

    #[wasm_bindgen(method, getter, js_namespace = BABYLON)]
    pub fn intensity(this: &Light) -> f64;

    #[wasm_bindgen(method, setter, js_namespace = BABYLON)]
    pub fn set_intensity(this: &Light, val: f64);

    #[wasm_bindgen(extends = Light)]
    #[derive(Debug)]
    pub type HemisphericLight;

    #[wasm_bindgen(constructor, js_namespace = BABYLON)]
    pub fn new(name: String, direction: Vector3, scene: &Scene) -> HemisphericLight;

    #[wasm_bindgen(extends = Light)]
    #[derive(Debug)]
    pub type PointLight;

    #[wasm_bindgen(constructor, js_namespace = BABYLON)]
    pub fn new(name: String, position: Vector3, scene: &Scene) -> PointLight;

    #[wasm_bindgen(extends = Light)]
    #[derive(Debug)]
    pub type SpotLight;

    #[wasm_bindgen(constructor, js_namespace = BABYLON)]
    pub fn new(
        name: String,
        position: Vector3,
        direction: Vector3,
        angle: f64,
        exponent: f64,
        scene: &Scene,
    ) -> SpotLight;

    #[derive(Debug)]
    pub type Mesh;

    #[wasm_bindgen(method, getter, js_namespace = BABYLON)]
    pub fn position(this: &Mesh) -> Option<Vector3>;

    #[wasm_bindgen(method, setter, js_namespace = BABYLON)]
    pub fn set_position(this: &Mesh, val: &Vector3);

    #[wasm_bindgen(method, getter, js_namespace = BABYLON)]
    pub fn material(this: &Mesh) -> Option<Material>;

    #[wasm_bindgen(method, setter, js_namespace = BABYLON)]
    pub fn set_material(this: &Mesh, val: &Material);

    #[derive(Debug)]
    pub type MeshBuilder;

    #[wasm_bindgen(static_method_of = MeshBuilder, js_name="CreateSphere", js_namespace = BABYLON)]
    pub fn create_sphere(name: String, options: CreateSphereOptions, scene: Option<&Scene>)
        -> Mesh;

    #[wasm_bindgen(static_method_of = MeshBuilder, js_name="CreateBox", js_namespace = BABYLON)]
    pub fn create_box(name: String, options: CreateBoxOptions, scene: Option<&Scene>) -> Mesh;

    #[wasm_bindgen(static_method_of = MeshBuilder, js_name="CreateCylinder", js_namespace = BABYLON)]
    pub fn create_cylinder(
        name: String,
        options: CreateCylinderOptions,
        scene: Option<&Scene>,
    ) -> Mesh;

    #[derive(Debug)]
    pub type TransformNode;

    #[derive(Debug)]
    pub type Texture;

    #[wasm_bindgen(constructor, js_namespace = BABYLON)]
    pub fn new(image_path: String, scene: &Scene) -> Texture;

    #[wasm_bindgen(method, getter, js_name="getAlphaFromRGB", js_namespace = BABYLON)]
    pub fn get_alpha_from_rgb(this: &Texture) -> bool;

    #[wasm_bindgen(method, setter, js_name="getAlphaFromRGB", js_namespace = BABYLON)]
    pub fn set_get_alpha_from_rgb(this: &Texture, val: bool);

    #[derive(Debug)]
    pub type Material;

    #[wasm_bindgen(extends = Material)]
    #[derive(Debug)]
    pub type StandardMaterial;

    #[wasm_bindgen(constructor, js_namespace = BABYLON)]
    pub fn new(name: String, scene: &Scene) -> StandardMaterial;

    #[wasm_bindgen(method, getter, js_name="diffuseTexture", js_namespace = BABYLON)]
    pub fn diffuse_texture(this: &StandardMaterial) -> Option<Texture>;

    #[wasm_bindgen(method, setter, js_name="diffuseTexture", js_namespace = BABYLON)]
    pub fn set_diffuse_texture(this: &StandardMaterial, val: &Texture);

    #[wasm_bindgen(method, getter, js_name="bumpTexture", js_namespace = BABYLON)]
    pub fn bump_texture(this: &StandardMaterial) -> Option<Texture>;

    #[wasm_bindgen(method, setter, js_name="bumpTexture", js_namespace = BABYLON)]
    pub fn set_bump_texture(this: &StandardMaterial, val: &Texture);

    #[wasm_bindgen(method, getter, js_name="opacityTexture", js_namespace = BABYLON)]
    pub fn opacity_texture(this: &StandardMaterial) -> Option<Texture>;

    #[wasm_bindgen(method, setter, js_name="opacityTexture", js_namespace = BABYLON)]
    pub fn set_opacity_texture(this: &StandardMaterial, val: &Texture);

    #[wasm_bindgen(extends = Material)]
    #[derive(Debug)]
    pub type PBRMaterial;

    #[wasm_bindgen(constructor, js_namespace = BABYLON)]
    pub fn new(name: String, scene: &Scene) -> PBRMaterial;

    #[wasm_bindgen(method, setter, js_name="useRoughnessFromMetallicTextureAlpha", js_namespace = BABYLON)]
    pub fn set_use_roughness_from_metallic_texture_alpha(this: &PBRMaterial, val: bool);

    #[wasm_bindgen(method, setter, js_name="useRoughnessFromMetallicTextureGreen", js_namespace = BABYLON)]
    pub fn set_use_roughness_from_metallic_texture_green(this: &PBRMaterial, val: bool);

    #[wasm_bindgen(method, setter, js_name="useMetallnessFromMetallicTextureBlue", js_namespace = BABYLON)]
    pub fn set_use_metallness_from_metallic_texture_blue(this: &PBRMaterial, val: bool);

    #[wasm_bindgen(method, setter, js_name="usePhysicalLightFalloff", js_namespace = BABYLON)]
    pub fn set_use_physical_light_falloff(this: &PBRMaterial, val: bool);

    #[wasm_bindgen(method, setter, js_name="albedoTexture", js_namespace = BABYLON)]
    pub fn set_albedo_texture(this: &PBRMaterial, val: &Texture);

    #[wasm_bindgen(method, setter, js_name="ambientTexture", js_namespace = BABYLON)]
    pub fn set_ambient_texture(this: &PBRMaterial, val: &Texture);

    #[wasm_bindgen(method, setter, js_name="bumpTexture", js_namespace = BABYLON)]
    pub fn set_bump_texture(this: &PBRMaterial, val: &Texture);

    #[wasm_bindgen(method, setter, js_name="metallicTexture", js_namespace = BABYLON)]
    pub fn set_metallic_texture(this: &PBRMaterial, val: &Texture);
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

#[wasm_bindgen]
#[derive(Debug, Default)]
pub struct CreateBoxOptions {
    pub backUVs: Option<bool>, // Vector4
    pub bottomBaseAt: Option<f64>,
    pub depth: Option<f64>,
    pub faceColors: Option<bool>, // Color4[]
    pub faceUV: Option<bool>,     // Vector4[]
    pub frontUVs: Option<bool>,   // Vector4
    pub height: Option<f64>,
    pub sideOrientation: Option<f64>,
    pub size: Option<f64>,
    pub topBaseAt: Option<f64>,
    pub updatable: Option<bool>,
    pub width: Option<f64>,
    pub wrap: Option<bool>,
}

#[wasm_bindgen]
#[derive(Debug, Default)]
pub struct CreateCylinderOptions {
    pub arc: Option<f64>,
    pub backUVs: Option<bool>, // Vector4
    pub cap: Option<f64>,
    pub diameter: Option<f64>,
    pub diameterBottom: Option<f64>,
    pub diameterTop: Option<f64>,
    pub enclose: Option<bool>,
    pub faceColors: Option<bool>, // Color4[]
    pub faceUV: Option<bool>,     // Vector4[]
    pub frontUVs: Option<bool>,   // Vector4
    pub hasRings: Option<bool>,
    pub height: Option<f64>,
    pub sideOrientation: Option<f64>,
    pub subdivisions: Option<f64>,
    pub tessellation: Option<f64>,
    pub updatable: Option<bool>,
}
