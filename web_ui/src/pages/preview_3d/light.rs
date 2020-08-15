use crate::{js::babylon, pages::preview_3d::Vector};

pub struct CreateLightArgs<'a> {
    pub scene: &'a babylon::Scene,
    pub lightbeam_falloff: &'a babylon::Material,
    pub origin_position: Vector,
}
pub fn create_light<'a>(args: CreateLightArgs<'a>) {
    let beam_angle = f64::to_radians(30.0);
    let cone_length = 50.0;
    let base_length = f64::tan(beam_angle / 2.0) * cone_length * 2.0;

    let cone = babylon::MeshBuilder::create_cylinder(
        "light_cone".to_string(),
        babylon::CreateCylinderOptions {
            height: Some(cone_length),
            diameterTop: Some(0.5),
            diameterBottom: Some(base_length),
            tessellation: Some(96.0),
            subdivisions: Some(4.0),
            enclose: Some(false),
            sideOrientation: Some(babylon::Mesh::doubleside()),
            ..Default::default()
        },
        Some(&args.scene),
    );
    cone.set_position(&babylon::Vector3::from(&args.origin_position));
    cone.set_material(&args.lightbeam_falloff);

    let spot_light = babylon::SpotLight::new(
        "spot_light".to_string(),
        babylon::Vector3::new(
            args.origin_position.x,
            args.origin_position.y + 25.5,
            args.origin_position.z,
        ),
        babylon::Vector3::new(0.0, -1.0, 0.0),
        beam_angle,
        1.0,
        &args.scene,
    );
    spot_light.set_intensity(8.0);
}
