use im_rc::HashMap;
use wasm_bindgen::prelude::*;
use yew::prelude::*;

use roller_protocol::fixture::{FixtureId, FixtureParams, FixtureState};

use crate::{console_log, js::babylon, yewtil::neq_assign::NeqAssign};

#[derive(Debug, Properties, Clone, PartialEq)]
pub struct PurePreview3dProps {
    pub fixture_states: HashMap<FixtureId, (FixtureParams, Option<FixtureState>)>,
}

#[derive(Debug)]
struct CanvasState {
    canvas_element: web_sys::HtmlCanvasElement,
    engine: babylon::Engine,
    scene: babylon::Scene,
    run_loop_closure: Closure<dyn FnMut()>,
}

#[derive(Debug)]
pub struct Preview3dPage {
    props: PurePreview3dProps,
    canvas_ref: NodeRef,
    canvas_state: Option<CanvasState>,
}

impl Component for Preview3dPage {
    type Message = ();
    type Properties = PurePreview3dProps;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        let canvas_ref = NodeRef::default();

        Preview3dPage {
            props,
            canvas_ref,
            canvas_state: None,
        }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props.neq_assign(props)
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            console_log!("{}", babylon::Engine::version());

            let canvas_element: web_sys::HtmlCanvasElement = self.canvas_ref.cast().unwrap();
            let engine = babylon::Engine::new(&canvas_element, Some(true), None, None);
            let scene = babylon::Scene::new(&engine);

            let concrete1 = babylon::StandardMaterial::new("concrete1".to_string(), &scene);
            concrete1.set_diffuse_texture(&babylon::Texture::new(
                "/assets/textures/Concrete-0202b.jpg".to_string(),
                &scene,
            ));
            concrete1.set_bump_texture(&babylon::Texture::new(
                "/assets/textures/Concrete-0202b-normal.png".to_string(),
                &scene,
            ));

            let concrete2 = babylon::StandardMaterial::new("concrete2".to_string(), &scene);
            concrete2.set_diffuse_texture(&babylon::Texture::new(
                "/assets/textures/Concrete-2341b.jpg".to_string(),
                &scene,
            ));
            concrete2.set_bump_texture(&babylon::Texture::new(
                "/assets/textures/Concrete-2341b-normal.jpg".to_string(),
                &scene,
            ));

            let concrete_floor = babylon::PBRMaterial::new("concrete_floor".to_string(), &scene);
            concrete_floor.set_albedo_texture(&babylon::Texture::new(
                "/assets/textures/concrete_rough_uhroebug/uhroebug_4K_Albedo.jpg".to_string(),
                &scene,
            ));
            concrete_floor.set_metallic_texture(&babylon::Texture::new(
                "/assets/textures/concrete_rough_uhroebug/uhroebug_4K_Roughness.jpg".to_string(),
                &scene,
            ));
            concrete_floor.set_bump_texture(&babylon::Texture::new(
                "/assets/textures/concrete_rough_uhroebug/uhroebug_4K_Bump.jpg".to_string(),
                &scene,
            ));
            concrete_floor.set_ambient_texture(&babylon::Texture::new(
                "/assets/textures/concrete_rough_uhroebug/uhroebug_4K_AO.jpg".to_string(),
                &scene,
            ));

            let wooden_floor = babylon::PBRMaterial::new("wooden_floor".to_string(), &scene);
            wooden_floor.set_albedo_texture(&babylon::Texture::new(
                "/assets/textures/wood_board_ugcwcevaw/ugcwcevaw_4K_Albedo.jpg".to_string(),
                &scene,
            ));
            wooden_floor.set_metallic_texture(&babylon::Texture::new(
                "/assets/textures/wood_board_ugcwcevaw/ugcwcevaw_4K_Roughness.jpg".to_string(),
                &scene,
            ));
            wooden_floor.set_bump_texture(&babylon::Texture::new(
                "/assets/textures/wood_board_ugcwcevaw/ugcwcevaw_4K_Normal.jpg".to_string(),
                &scene,
            ));
            wooden_floor.set_ambient_texture(&babylon::Texture::new(
                "/assets/textures/wood_board_ugcwcevaw/ugcwcevaw_4K_AO.jpg".to_string(),
                &scene,
            ));
            wooden_floor.set_use_physical_light_falloff(false);
            wooden_floor.set_use_roughness_from_metallic_texture_alpha(false);
            wooden_floor.set_use_roughness_from_metallic_texture_green(true);
            wooden_floor.set_use_metallness_from_metallic_texture_blue(true);

            let lightbeam_falloff1 =
                babylon::StandardMaterial::new("lightbeam_falloff1".to_string(), &scene);
            lightbeam_falloff1.set_opacity_texture(&{
                let texture = babylon::Texture::new(
                    "/assets/textures/lightbeam_falloff1.jpg".to_string(),
                    &scene,
                );
                texture.set_get_alpha_from_rgb(true);
                texture
            });

            let camera = babylon::ArcRotateCamera::new(
                "Camera".to_string(),
                std::f64::consts::PI / 2.0,
                std::f64::consts::PI / 2.0,
                2.0,
                babylon::Vector3::new(0.0, 0.0, 5.0),
                &scene,
                None,
            );
            camera.attach_control(&canvas_element, None, None, None);

            // let light1 = babylon::HemisphericLight::new(
            //     "light1".to_string(),
            //     babylon::Vector3::new(1.0, 50.0, 0.0),
            //     &scene,
            // );
            // let light2 = babylon::PointLight::new(
            //     "light2".to_string(),
            //     babylon::Vector3::new(0.0, 1.0, -1.0),
            //     &scene,
            // );
            let light3 = babylon::SpotLight::new(
                "light3".to_string(),
                babylon::Vector3::new(0.0, 30.0, -5.0),
                babylon::Vector3::new(0.0, -1.0, 0.0),
                std::f64::consts::PI / 3.0,
                1.5,
                &scene,
            );
            let light3: &babylon::Light = light3.as_ref();
            light3.set_intensity(5.0);

            let sphere = babylon::MeshBuilder::create_sphere(
                "sphere".to_string(),
                babylon::CreateSphereOptions {
                    diameter: Some(2.0),
                    ..Default::default()
                },
                Some(&scene),
            );
            let box1 = babylon::MeshBuilder::create_box(
                "box1".to_string(),
                babylon::CreateBoxOptions {
                    size: Some(2.0),
                    ..Default::default()
                },
                Some(&scene),
            );
            box1.set_material(&concrete2);
            let box2 = babylon::MeshBuilder::create_box(
                "box2".to_string(),
                babylon::CreateBoxOptions {
                    size: Some(2.0),
                    ..Default::default()
                },
                Some(&scene),
            );
            box2.set_material(&concrete2);

            box1.set_position(&babylon::Vector3::new(-5.0, 0.0, 0.0));
            box2.set_position(&babylon::Vector3::new(5.0, 0.0, 0.0));

            let cone = babylon::MeshBuilder::create_cylinder(
                "cone1".to_string(),
                babylon::CreateCylinderOptions {
                    height: Some(3.0),
                    diameterTop: Some(0.5),
                    diameterBottom: Some(3.0),
                    tessellation: Some(96.0),
                    subdivisions: Some(4.0),
                    ..Default::default()
                },
                Some(&scene),
            );
            cone.set_position(&babylon::Vector3::new(5.0, 1.0, 5.0));
            cone.set_material(&lightbeam_falloff1);

            let floor = babylon::MeshBuilder::create_box(
                "floor".to_string(),
                babylon::CreateBoxOptions {
                    width: Some(20.0),
                    depth: Some(20.0),
                    height: Some(0.1),
                    ..Default::default()
                },
                Some(&scene),
            );
            floor.set_material(&wooden_floor);
            floor.set_position(&babylon::Vector3::new(0.0, -2.0, 0.0));

            let scene1 = scene.clone();
            let run_loop_closure = Closure::new(move || {
                scene1.render(None, None);
            });
            engine.run_render_loop(&run_loop_closure);

            self.canvas_state = Some(CanvasState {
                canvas_element,
                engine,
                scene,
                run_loop_closure,
            });

            console_log!("{:?}", self.canvas_state);
        }
    }

    fn view(&self) -> Html {
        html! {
            <div class="page-contents">
                <h2>{"Fixtures 3D"}</h2>
                <div>
                    <canvas id="preview-3d-canvas" ref=self.canvas_ref.clone()></canvas>
                </div>
            </div>
        }
    }
}
