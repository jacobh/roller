use im_rc::HashMap;
use itertools::Itertools;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use yew::prelude::*;

use roller_protocol::{
    clock::Clock,
    fixture::{FixtureGroupId, FixtureId, FixtureLocation, FixtureParams, FixtureState},
    lighting_engine::FixtureGroupState,
};

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
impl From<&Vector> for babylon::Vector3 {
    fn from(vector: &Vector) -> babylon::Vector3 {
        babylon::Vector3::new(vector.x, vector.y, vector.z)
    }
}

#[derive(Debug, Properties, Clone, PartialEq)]
pub struct PurePreview3dProps {
    pub fixture_states: HashMap<FixtureId, (FixtureParams, Option<FixtureState>)>,
    pub clock: Rc<Clock>,
    pub base_fixture_group_state: Rc<FixtureGroupState>,
    pub fixture_group_states: HashMap<FixtureGroupId, FixtureGroupState>,
}

#[derive(Debug)]
struct CanvasState {
    canvas_element: web_sys::HtmlCanvasElement,
    engine: babylon::Engine,
    scene: babylon::Scene,
    run_loop_closure: Closure<dyn FnMut()>,
    lights: HashMap<FixtureId, light::Light>,
    hemispheric_light: babylon::HemisphericLight,
}

#[derive(Debug)]
pub struct Preview3dPage {
    props: PurePreview3dProps,
    canvas_ref: NodeRef,
    canvas_state: Option<CanvasState>,
    tick: Option<gloo::timers::callback::Interval>,
}

impl Component for Preview3dPage {
    type Message = ();
    type Properties = PurePreview3dProps;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        let canvas_ref = NodeRef::default();

        Preview3dPage {
            props,
            canvas_ref,
            tick: None,
            canvas_state: None,
        }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        let changed = self.props.neq_assign(props);

        if let Some(canvas_state) = self.canvas_state.as_mut() {
            let fixtures: Vec<(&FixtureParams, &FixtureState)> = self
                .props
                .fixture_states
                .values()
                .filter_map(|(params, state)| match state {
                    Some(state) => Some((params, state)),
                    None => None,
                })
                .collect();

            apply_fixture_states_to_canvas(&fixtures, canvas_state);
        }

        false
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            console_log!("{}", babylon::Engine::version());

            let positioned_fixtures: Vec<(FixtureId, FixtureLocation)> = self
                .props
                .fixture_states
                .iter()
                .filter_map(
                    |(id, (fixture_params, _))| match fixture_params.location.as_ref() {
                        Some(location) => Some((*id, location.clone())),
                        None => None,
                    },
                )
                .unique_by(|(_, loc)| loc.clone())
                .collect();

            let canvas_element: web_sys::HtmlCanvasElement = self.canvas_ref.cast().unwrap();
            let engine = babylon::Engine::new(&canvas_element, Some(true), None, Some(true));
            let scene = babylon::Scene::new(&engine);
            scene.set_clear_color(babylon::Vector4::new(0.0, 0.0, 0.0, 1.0));
            scene.set_fog_mode(babylon::Scene::get_fog_mode_exp());
            scene.set_fog_color(babylon::Color3::new(0.1, 0.1, 0.1));
            scene.set_fog_density(0.0025);

            let concrete_floor = materials::load_concrete_floor(&scene);
            let concrete_wall = materials::load_concrete_wall(&scene);
            // let wooden_floor = materials::load_wooden_floor(&scene);
            let black_fabric = materials::load_black_fabric(&scene);
            let galaxy_marble = materials::load_black_galaxy_marble(&scene);

            let camera = babylon::UniversalCamera::new(
                "Camera".to_string(),
                babylon::Vector3::new(0.0, 2.0, 35.0),
                &scene,
            );
            camera.attach_control(&canvas_element, Some(true));
            camera.set_keys_up(vec![87]); // W
            camera.set_keys_left(vec![65]); // A
            camera.set_keys_down(vec![83]); // S
            camera.set_keys_right(vec![68]); // D
            camera.set_check_collisions(true);
            camera.set_apply_gravity(true);
            camera.set_ellipsoid(&babylon::Vector3::new(1.5, 1.5, 1.5));
            camera.set_speed(0.5);
            camera.set_target(babylon::Vector3::new(0.0, 0.0, 0.0));

            let hemispheric_light = babylon::HemisphericLight::new(
                "hemispheric_light".to_string(),
                babylon::Vector3::new(1.0, 50.0, 0.0),
                &scene,
            );
            hemispheric_light.set_intensity(0.1);
            // let light2 = babylon::PointLight::new(
            //     "light2".to_string(),
            //     babylon::Vector3::new(0.0, 15.0, -5.0),
            //     &scene,
            // );

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
                floor_material: &galaxy_marble,
                width: 75.0,
                depth: 100.0,
                height: 30.0,
            });

            let mut lights = HashMap::new();
            for (id, location) in positioned_fixtures {
                let x = location.x as f64;
                let y = location.y as f64;
                let light = light::create_light(light::CreateLightArgs {
                    scene: &scene,
                    origin_position: Vector::new(x * 2.0 - 50.0, 15.0, y * 2.0 - 50.0),
                });
                lights.insert(id, light);
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
                lights,
                hemispheric_light,
            });

            self.tick = Some(gloo::timers::callback::Interval::new(1_000, || {
                console_log!("hello...");
            }));

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

fn apply_fixture_states_to_canvas(
    fixtures: &[(&FixtureParams, &FixtureState)],
    canvas_state: &mut CanvasState,
) {
    // very crudely light the entire room by adding up all the dimmer values
    let total_dimmer: f64 = fixtures.iter().map(|(_, state)| state.dimmer).sum();

    canvas_state
        .hemispheric_light
        .set_intensity(0.05 + total_dimmer / 40.0);
    canvas_state.scene.set_fog_color(babylon::Color3::new(
        total_dimmer / 50.0,
        total_dimmer / 50.0,
        total_dimmer / 50.0,
    ));

    for (params, state) in fixtures.iter() {
        let light = canvas_state.lights.get_mut(&params.id);

        if let Some(light) = light {
            light.set_dimmer(state.dimmer);
        }
    }
}
