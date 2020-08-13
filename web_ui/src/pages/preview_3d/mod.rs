use im_rc::HashMap;
use wasm_bindgen::prelude::*;
use yew::prelude::*;

use roller_protocol::fixture::{FixtureId, FixtureParams, FixtureState};

use crate::{console_log, js::babylon, yewtil::neq_assign::NeqAssign};

mod materials;

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

            let concrete_floor = materials::load_concrete_floor(&scene);
            let concrete_wall = materials::load_concrete_wall(&scene);
            let wooden_floor = materials::load_wooden_floor(&scene);
            let lightbeam_falloff = materials::load_lightbeam_falloff(&scene);
            let black_fabric = materials::load_black_fabric(&scene);

            let camera = babylon::UniversalCamera::new(
                "Camera".to_string(),
                babylon::Vector3::new(0.0, 0.0, 5.0),
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
            let light3: &babylon::Light = light3.as_ref();
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
            floor.set_check_collisions(true);
            floor.set_material(&wooden_floor);
            floor.set_position(&babylon::Vector3::new(0.0, -2.0, 0.0));

            let front_wall = babylon::MeshBuilder::create_box(
                "front_wall".to_string(),
                babylon::CreateBoxOptions {
                    width: Some(20.0),
                    depth: Some(0.1),
                    height: Some(10.0),
                    ..Default::default()
                },
                Some(&scene),
            );
            front_wall.set_check_collisions(true);
            front_wall.set_material(&black_fabric);
            front_wall.set_position(&babylon::Vector3::new(0.0, 3.0, -10.0));

            let left_wall = babylon::MeshBuilder::create_box(
                "left_wall".to_string(),
                babylon::CreateBoxOptions {
                    width: Some(0.1),
                    depth: Some(20.0),
                    height: Some(10.0),
                    ..Default::default()
                },
                Some(&scene),
            );
            left_wall.set_check_collisions(true);
            left_wall.set_material(&concrete_wall);
            left_wall.set_position(&babylon::Vector3::new(-10.0, 3.0, 0.0));

            let right_wall = babylon::MeshBuilder::create_box(
                "right_wall".to_string(),
                babylon::CreateBoxOptions {
                    width: Some(0.1),
                    depth: Some(20.0),
                    height: Some(10.0),
                    ..Default::default()
                },
                Some(&scene),
            );
            right_wall.set_check_collisions(true);
            right_wall.set_material(&concrete_wall);
            right_wall.set_position(&babylon::Vector3::new(10.0, 3.0, 0.0));

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
