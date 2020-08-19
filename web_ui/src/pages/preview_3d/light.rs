use crate::{
    console_log,
    js::babylon,
    pages::preview_3d::{materials::load_lightbeam_falloff, Vector},
};

const SPOT_LIGHT_MAX_INTENSITY: f64 = 10.0;

#[derive(Debug, Clone)]
pub struct Light {
    cone_material: babylon::StandardMaterial,
    cone_mesh: babylon::Mesh,
    spot_light: babylon::SpotLight,
    dimmer: f64,
    color: (f64, f64, f64),
}
impl Light {
    pub fn set_dimmer(&mut self, dimmer: f64) {
        if self.dimmer != dimmer {
            self.dimmer = dimmer;
            self.spot_light
                .set_intensity(SPOT_LIGHT_MAX_INTENSITY * dimmer);
            self.cone_material.set_alpha(dimmer);
        }
    }
    pub fn set_color(&mut self, color: (f64, f64, f64)) {
        if self.color != color {
            self.color = color;

            let babylon_color = babylon::Color3::new(color.0, color.1, color.2);

            self.cone_material.set_emissive_color(&babylon_color);

            self.spot_light.set_diffuse(babylon_color);
        }
    }
}

pub struct CreateLightArgs<'a> {
    pub scene: &'a babylon::Scene,
    pub origin_position: Vector,
}
pub fn create_light<'a>(args: CreateLightArgs<'a>) -> Light {
    let beam_angle = f64::to_radians(30.0);
    let cone_length = 50.0;
    let base_length = f64::tan(beam_angle / 2.0) * cone_length * 2.0;

    let cone_material = load_lightbeam_falloff(args.scene);
    let cone_mesh = babylon::MeshBuilder::create_cylinder(
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
    cone_mesh.set_position(&babylon::Vector3::from(&args.origin_position));
    cone_mesh.set_material(&cone_material);

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
    spot_light.set_intensity(SPOT_LIGHT_MAX_INTENSITY);

    Light {
        spot_light,
        cone_material,
        cone_mesh,
        dimmer: 1.0,
        color: (1.0, 1.0, 1.0),
    }
}
