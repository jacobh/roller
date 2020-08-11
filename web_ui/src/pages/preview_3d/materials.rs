use crate::js::babylon;

pub fn load_concrete_floor(scene: &babylon::Scene) -> babylon::PBRMaterial {
    let concrete_floor = babylon::PBRMaterial::new("concrete_floor".to_string(), &scene);
    concrete_floor.set_albedo_texture(&babylon::Texture::new(
        "/assets/textures/concrete_rough_uhroebug/uhroebug_4K_Albedo.jpg".to_string(),
        &scene,
    ));
    concrete_floor.set_reflectivity_texture(&babylon::Texture::new(
        "/assets/textures/concrete_rough_uhroebug/uhroebug_4K_Specular.jpg".to_string(),
        &scene,
    ));
    concrete_floor.set_micro_surface_texture(&babylon::Texture::new(
        "/assets/textures/concrete_rough_uhroebug/uhroebug_4K_Gloss.jpg".to_string(),
        &scene,
    ));
    concrete_floor.set_bump_texture(&babylon::Texture::new(
        "/assets/textures/concrete_rough_uhroebug/uhroebug_4K_Normal.jpg".to_string(),
        &scene,
    ));
    concrete_floor.set_ambient_texture(&babylon::Texture::new(
        "/assets/textures/concrete_rough_uhroebug/uhroebug_4K_AO.jpg".to_string(),
        &scene,
    ));
    concrete_floor.set_use_physical_light_falloff(false);

    concrete_floor
}

pub fn load_wooden_floor(scene: &babylon::Scene) -> babylon::PBRMaterial {
    let wooden_floor = babylon::PBRMaterial::new("wooden_floor".to_string(), &scene);
    wooden_floor.set_albedo_texture(&babylon::Texture::new(
        "/assets/textures/wood_board_ugcwcevaw/ugcwcevaw_4K_Albedo.jpg".to_string(),
        &scene,
    ));
    wooden_floor.set_reflectivity_texture(&babylon::Texture::new(
        "/assets/textures/wood_board_ugcwcevaw/ugcwcevaw_4K_Specular.jpg".to_string(),
        &scene,
    ));
    wooden_floor.set_micro_surface_texture(&babylon::Texture::new(
        "/assets/textures/wood_board_ugcwcevaw/ugcwcevaw_4K_Gloss.jpg".to_string(),
        &scene,
    ));
    wooden_floor.set_bump_texture(&babylon::Texture::new(
        "/assets/textures/wood_board_ugcwcevaw/ugcwcevaw_4K_Normal.jpg".to_string(),
        &scene,
    ));
    wooden_floor.set_ambient_texture(&babylon::Texture::new(
        "/assets/textures/wood_board_ugcwcevaw/ugcwcevaw_4K_AO.jpg".to_string(),
        &scene,
    ));
    wooden_floor.set_use_physical_light_falloff(false);

    wooden_floor
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
