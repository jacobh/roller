use crate::js::babylon;

fn load_megascans_material(slug: &str, id: &str, scene: &babylon::Scene) -> babylon::PBRMaterial {
    let material = babylon::PBRMaterial::new(format!("{}_{}", slug, id), &scene);

    material.set_albedo_texture(&babylon::Texture::new(
        format!("/assets/textures/{}_{}/{}_4K_Albedo.jpg", slug, id, id),
        &scene,
    ));
    material.set_reflectivity_texture(&babylon::Texture::new(
        format!("/assets/textures/{}_{}/{}_4K_Specular.jpg", slug, id, id),
        &scene,
    ));
    material.set_micro_surface_texture(&babylon::Texture::new(
        format!("/assets/textures/{}_{}/{}_4K_Gloss.jpg", slug, id, id),
        &scene,
    ));
    material.set_bump_texture(&babylon::Texture::new(
        format!("/assets/textures/{}_{}/{}_4K_Normal.jpg", slug, id, id),
        &scene,
    ));
    material.set_ambient_texture(&babylon::Texture::new(
        format!("/assets/textures/{}_{}/{}_4K_AO.jpg", slug, id, id),
        &scene,
    ));
    material.set_use_physical_light_falloff(false);

    material
}

pub fn load_concrete_floor(scene: &babylon::Scene) -> babylon::PBRMaterial {
    load_megascans_material("concrete_rough", "uhroebug", scene)
}

pub fn load_wooden_floor(scene: &babylon::Scene) -> babylon::PBRMaterial {
    load_megascans_material("wood_board", "ugcwcevaw", scene)
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
