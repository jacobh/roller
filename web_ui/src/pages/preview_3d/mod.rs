use im_rc::HashMap;
use wasm_bindgen::prelude::*;
use yew::prelude::*;

use roller_protocol::fixture::{FixtureId, FixtureParams, FixtureState};

use crate::{console_log, js::babylon, yewtil::neq_assign::NeqAssign};

mod light;
mod materials;
mod room;

pub struct Vector {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}
impl Vector {
    pub fn new(x: f64, y: f64, z: f64) -> Vector {
        Vector { x, y, z }
    }
}
impl From<Vector> for babylon::Vector3 {
    fn from(vector: Vector) -> babylon::Vector3 {
        babylon::Vector3::new(vector.x, vector.y, vector.z)
    }
}

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

            let light_positions: Vec<_> = self
                .props
                .fixture_states
                .values()
                .filter_map(|(fixture_params, _)| fixture_params.location.as_ref())
                .collect();

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

            room::create_room(room::CreateRoomArgs {
                scene: &scene,
                front_wall_material: &black_fabric,
                back_wall_material: &black_fabric,
                left_wall_material: &concrete_wall,
                right_wall_material: &concrete_wall,
                floor_material: &wooden_floor,
                width: 75.0,
                depth: 100.0,
                height: 30.0,
            });

            for light_position in light_positions {
                light::create_light(light::CreateLightArgs {
                    scene: &scene,
                    lightbeam_falloff: &lightbeam_falloff,
                    origin_position: Vector::new(
                        light_position.x as f64,
                        15.0,
                        light_position.y as f64,
                    ),
                });
            }

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
