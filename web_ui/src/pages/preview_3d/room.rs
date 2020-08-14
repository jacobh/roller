use wasm_bindgen::prelude::*;
// use yew::prelude::*;

use crate::{console_log, js::babylon};

struct CreateFaceArgs<'a> {
    name: &'a str,
    scene: &'a babylon::Scene,
    material: &'a babylon::Material,
    width: f64,
    depth: f64,
    height: f64,
}
fn create_face<'a>(args: CreateFaceArgs<'a>) -> babylon::Mesh {
    let face = babylon::MeshBuilder::create_box(
        args.name.to_string(),
        babylon::CreateBoxOptions {
            width: Some(args.width),
            depth: Some(args.depth),
            height: Some(args.height),
            ..Default::default()
        },
        Some(args.scene),
    );
    face.set_check_collisions(true);
    face.set_material(args.material);
    face
}

const FLOOR_HEIGHT: f64 = -2.0;

pub struct CreateRoomArgs<'a> {
    pub scene: &'a babylon::Scene,
    pub front_wall_material: &'a babylon::Material,
    pub back_wall_material: &'a babylon::Material,
    pub left_wall_material: &'a babylon::Material,
    pub right_wall_material: &'a babylon::Material,
    pub floor_material: &'a babylon::Material,
    // width: f64,
    // depth: f64,
    // height: f64,
}
pub fn create_room<'a>(args: CreateRoomArgs<'a>) {
    let floor = create_face(CreateFaceArgs {
        scene: args.scene,
        name: "floor",
        material: args.floor_material,
        width: 75.0,
        depth: 100.0,
        height: 0.1,
    });
    floor.set_position(&babylon::Vector3::new(0.0, -2.0, 0.0));

    let front_wall = create_face(CreateFaceArgs {
        scene: args.scene,
        name: "front_wall",
        material: args.front_wall_material,
        width: 75.0,
        depth: 0.1,
        height: 35.0,
    });
    front_wall.set_position(&babylon::Vector3::new(0.0, 15.0, -50.0));

    let back_wall = create_face(CreateFaceArgs {
        scene: args.scene,
        name: "back_wall",
        material: args.back_wall_material,
        width: 75.0,
        depth: 0.1,
        height: 35.0,
    });
    back_wall.set_position(&babylon::Vector3::new(0.0, 15.0, 50.0));

    let left_wall = create_face(CreateFaceArgs {
        scene: args.scene,
        name: "left_wall",
        material: args.left_wall_material,
        width: 0.1,
        depth: 100.0,
        height: 35.0,
    });
    left_wall.set_position(&babylon::Vector3::new(-37.5, 15.0, 0.0));

    let right_wall = create_face(CreateFaceArgs {
        scene: args.scene,
        name: "right_wall",
        material: args.right_wall_material,
        width: 0.1,
        depth: 100.0,
        height: 35.0,
    });
    right_wall.set_position(&babylon::Vector3::new(37.5, 15.0, 0.0));
}
