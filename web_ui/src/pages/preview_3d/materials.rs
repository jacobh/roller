use crate::js::babylon;

fn load_texture(name: String, scale: f64, scene: &babylon::Scene) -> babylon::Texture {
    let texture = babylon::Texture::new(name, &scene);
    texture.set_u_scale(scale);
    texture.set_v_scale(scale);

    texture
}

fn load_megascans_material(
    slug: &str,
    id: &str,
    scale: f64,
    scene: &babylon::Scene,
) -> babylon::PBRMaterial {
    let material = babylon::PBRMaterial::new(format!("{}_{}", slug, id), &scene);

    material.set_albedo_texture(&load_texture(
        format!("/assets/textures/{}_{}/{}_4K_Albedo.jpg", slug, id, id),
        scale,
        &scene,
    ));
    material.set_metallic_texture(&load_texture(
        format!(
            "/assets/textures/{}_{}/{}_4K_MetalRoughness.jpg",
            slug, id, id
        ),
        scale,
        &scene,
    ));
    material.set_bump_texture(&load_texture(
        format!("/assets/textures/{}_{}/{}_4K_Normal.jpg", slug, id, id),
        scale,
        &scene,
    ));
    material.set_use_ambient_occlusion_from_metallic_texture_red(true);
    material.set_use_roughness_from_metallic_texture_green(true);
    material.set_use_metallness_from_metallic_texture_blue(true);
    material.set_use_physical_light_falloff(false);

    material
}

pub fn load_concrete_floor(scene: &babylon::Scene) -> babylon::PBRMaterial {
    load_megascans_material("concrete_rough", "uhroebug", 1.0, scene)
}

pub fn load_concrete_wall(scene: &babylon::Scene) -> babylon::PBRMaterial {
    load_megascans_material("concrete_rough", "ugxkfj0dy", 1.0, scene)
}

pub fn load_wooden_floor(scene: &babylon::Scene) -> babylon::PBRMaterial {
    load_megascans_material("wood_board", "ugcwcevaw", 1.0, scene)
}

pub fn load_black_fabric(scene: &babylon::Scene) -> babylon::PBRMaterial {
    load_megascans_material("fabric_plain", "pgjeuxp0", 1.0, scene)
}

pub fn load_lightbeam_falloff(scene: &babylon::Scene) -> babylon::StandardMaterial {
    let lightbeam_falloff =
        babylon::StandardMaterial::new("lightbeam_falloff1".to_string(), &scene);
    lightbeam_falloff.set_opacity_texture(&{
        let texture = babylon::Texture::new(
            "/assets/textures/lightbeam_falloff1.jpg".to_string(),
            &scene,
        );
        texture.set_get_alpha_from_rgb(true);
        texture
    });

    lightbeam_falloff
}
