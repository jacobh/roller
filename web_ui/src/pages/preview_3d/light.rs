use crate::js::babylon;

pub struct CreateLightArgs<'a> {
    pub scene: &'a babylon::Scene,
    pub lightbeam_falloff: &'a babylon::Material,
}
pub fn create_light<'a>(args: CreateLightArgs<'a>) {
    let cone = babylon::MeshBuilder::create_cylinder(
        "light_cone".to_string(),
        babylon::CreateCylinderOptions {
            height: Some(30.0),
            diameterTop: Some(0.5),
            diameterBottom: Some(10.0),
            tessellation: Some(96.0),
            subdivisions: Some(4.0),
            enclose: false,
            ..Default::default()
        },
        Some(&args.scene),
    );
    cone.set_position(&babylon::Vector3::new(10.0, 15.0, -35.0));
    cone.set_material(&args.lightbeam_falloff);
}
