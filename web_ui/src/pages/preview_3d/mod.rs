use im_rc::HashMap;
use wasm_bindgen::prelude::*;
use yew::prelude::*;

use roller_protocol::fixture::{FixtureId, FixtureParams, FixtureState};

use crate::{console_log, js::babylon, yewtil::neq_assign::NeqAssign};

mod materials;
mod room;

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
            let engine = babylon::Engine::new(&canvas_element, Some(true), None, Some(true));
            let scene = babylon::Scene::new(&engine);
            scene.set_clear_color(babylon::Vector4::new(0.0, 0.0, 0.0, 1.0));

            let concrete_floor = materials::load_concrete_floor(&scene);
            let concrete_wall = materials::load_concrete_wall(&scene);
            let wooden_floor = materials::load_wooden_floor(&scene);
            let lightbeam_falloff = materials::load_lightbeam_falloff(&scene);
            let black_fabric = materials::load_black_fabric(&scene);

            let camera = babylon::UniversalCamera::new(
                "Camera".to_string(),
                babylon::Vector3::new(0.0, 2.0, 35.0),
                &scene,
            );
            camera.attach_control(&canvas_element, Some(true));
            camera.set_keys_up(&[87]); // W
            camera.set_keys_left(&[65]); // A
            camera.set_keys_down(&[83]); // S
            camera.set_keys_right(&[68]); // D
            camera.set_check_collisions(true);
            camera.set_apply_gravity(true);
            camera.set_ellipsoid(&babylon::Vector3::new(1.5, 1.5, 1.5));
            camera.set_speed(0.5);
            camera.set_target(babylon::Vector3::new(0.0, 0.0, 0.0));

            // let light1 = babylon::HemisphericLight::new(
            //     "light1".to_string(),
            //     babylon::Vector3::new(1.0, 50.0, 0.0),
            //     &scene,
            // );
            let light2 = babylon::PointLight::new(
                "light2".to_string(),
                babylon::Vector3::new(0.0, 15.0, -5.0),
                &scene,
            );
            let light3 = babylon::SpotLight::new(
                "light3".to_string(),
                babylon::Vector3::new(0.0, 15.0, -5.0),
                babylon::Vector3::new(0.0, -1.0, 0.0),
                std::f64::consts::PI / 3.0,
                3.0,
                &scene,
            );
            light3.set_intensity(10.0);

            let sphere = babylon::MeshBuilder::create_sphere(
                "sphere".to_string(),
                babylon::CreateSphereOptions {
                    diameter: Some(2.0),
                    ..Default::default()
                },
                Some(&scene),
            );
            sphere.set_check_collisions(true);
            let box1 = babylon::MeshBuilder::create_box(
                "box1".to_string(),
                babylon::CreateBoxOptions {
                    size: Some(2.0),
                    ..Default::default()
                },
                Some(&scene),
            );
            box1.set_material(&concrete_floor);
            box1.set_check_collisions(true);
            let box2 = babylon::MeshBuilder::create_box(
                "box2".to_string(),
                babylon::CreateBoxOptions {
                    size: Some(2.0),
                    ..Default::default()
                },
                Some(&scene),
            );
            box2.set_material(&black_fabric);
            box2.set_check_collisions(true);

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
            cone.set_material(&lightbeam_falloff);

            room::create_room(room::CreateRoomArgs {
                scene: &scene,
                front_wall_material: &black_fabric,
                back_wall_material: &black_fabric,
                left_wall_material: &concrete_wall,
                right_wall_material: &concrete_wall,
                floor_material: &wooden_floor,
                // width: 75,
                // depth: 100.0,
                // height: 0.1,
            });

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
