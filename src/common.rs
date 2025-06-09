use bevy::prelude::*;

#[derive(Resource)]
pub struct Common {
    pub mesh_cube: Handle<Mesh>,
    pub mesh_ruby: Handle<Mesh>,
    pub mesh_sphere: Handle<Mesh>,
    pub mesh_cylinder: Handle<Mesh>,
    pub mesh_plane: Handle<Mesh>,
    pub mesh_small_sphere: Handle<Mesh>,
    pub material_gray: Handle<StandardMaterial>,
    pub material_pink: Handle<StandardMaterial>,
    pub material_dark_gray: Handle<StandardMaterial>,
    pub material_yellow: Handle<StandardMaterial>,
    pub material_dark_blue: Handle<StandardMaterial>,
    pub material_orange: Handle<StandardMaterial>,
    pub material_beepboop: Handle<StandardMaterial>,
    pub material_active: Handle<StandardMaterial>,
    pub material_electricity: Handle<StandardMaterial>,
    pub material_laser: Handle<StandardMaterial>,
    pub material_invisible: Handle<StandardMaterial>,
    pub material_fog: Handle<StandardMaterial>,
    pub material_outlet: Handle<StandardMaterial>,
    pub material_zappy_boy: Handle<StandardMaterial>,
    pub material_zappy_field: Handle<StandardMaterial>,
    pub material_ruby: Handle<StandardMaterial>,

    pub image_e: Handle<Image>,
    pub image_fwd: Handle<Image>,

    pub material_icon_e: Handle<StandardMaterial>,
    pub material_icon_low_power: Handle<StandardMaterial>,

    pub scene_computer: Handle<Scene>,
    pub scene_ruby: Handle<Scene>,

    pub material_tutorial_move: Handle<StandardMaterial>,
    pub material_tutorial_interact: Handle<StandardMaterial>,
    pub material_tutorial_zip: Handle<StandardMaterial>,

    pub material_you_win: Handle<StandardMaterial>,
    pub material_email: Handle<StandardMaterial>,
}

#[derive(Default)]
pub struct CommonPlugin;

impl Plugin for CommonPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_common);
    }
}

/// Inserts the `Common` resource.
pub fn setup_common(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let image_e: Handle<Image> = asset_server.load("icon_e.png");
    let image_low_power: Handle<Image> = asset_server.load("icon_low_power.png");

    commands.insert_resource(Common {
        mesh_cube: meshes.add(Cuboid::default()),
        mesh_ruby: meshes.add(Cone::default()),
        mesh_plane: meshes.add(Plane3d::new(-Vec3::Z, Vec2::new(0.5, 0.5))),
        mesh_sphere: meshes.add(Sphere::default()),
        mesh_cylinder: meshes.add(Cylinder::default()),
        mesh_small_sphere: meshes.add(Sphere::new(0.2)),
        material_gray: materials.add(StandardMaterial {
            base_color: Color::linear_rgb(0.4, 0.5, 0.6),
            perceptual_roughness: 1.0,
            ..default()
        }),
        material_dark_gray: materials.add(StandardMaterial {
            base_color: Color::linear_rgb(0.2, 0.2, 0.3),
            perceptual_roughness: 1.0,
            ..default()
        }),
        material_yellow: materials.add(StandardMaterial {
            base_color: Color::linear_rgb(0.9, 0.8, 0.2),
            perceptual_roughness: 1.0,
            ..default()
        }),
        material_pink: materials.add(StandardMaterial {
            base_color: Color::linear_rgb(1.0, 0.5, 0.7),
            perceptual_roughness: 1.0,
            ..default()
        }),
        material_ruby: materials.add(StandardMaterial {
            base_color: Color::linear_rgb(1.0, 0.1, 0.2),
            perceptual_roughness: 0.2,
            reflectance: 0.3,
            ..default()
        }),
        material_dark_blue: materials.add(StandardMaterial {
            base_color: Color::linear_rgb(0.0, 0.05, 0.1),
            perceptual_roughness: 1.0,
            ..default()
        }),
        material_orange: materials.add(StandardMaterial {
            base_color: Color::linear_rgb(0.9, 0.4, 0.1),
            perceptual_roughness: 1.0,
            ..default()
        }),
        material_beepboop: materials.add(StandardMaterial {
            base_color: Color::linear_rgb(0.9, 0.87, 0.94),
            perceptual_roughness: 0.05,
            metallic: 1.0,
            ..default()
        }),
        material_active: materials.add(StandardMaterial {
            base_color: Color::linear_rgb(0.4, 0.5, 1.0),
            emissive: LinearRgba::rgb(0.3, 0.6, 1.0) * 20.,
            perceptual_roughness: 1.0,
            ..default()
        }),
        material_electricity: materials.add(StandardMaterial {
            base_color: Color::linear_rgb(0.4, 0.8, 1.0),
            emissive: LinearRgba::rgb(0.3, 0.8, 1.0) * 20.,
            perceptual_roughness: 1.0,
            ..default()
        }),
        material_laser: materials.add(StandardMaterial {
            base_color: Color::linear_rgb(1.0, 0.0, 0.3),
            emissive: LinearRgba::rgb(1.0, 0.2, 0.3) * 20.,
            perceptual_roughness: 1.0,
            ..default()
        }),
        material_zappy_boy: materials.add(StandardMaterial {
            base_color: Color::linear_rgb(0.8, 0.88, 0.4),
            emissive: LinearRgba::rgb(1.0, 0.2, 0.3) * 20.,
            perceptual_roughness: 1.0,
            metallic: 1.0,
            ..default()
        }),
        material_zappy_field: materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load("zappy.png")),
            alpha_mode: AlphaMode::Mask(0.5),
            emissive: LinearRgba::rgb(1.0, 1.0, 0.3) * 20.,
            perceptual_roughness: 1.0,
            cull_mode: None,
            ..default()
        }),
        material_invisible: materials.add(StandardMaterial {
            base_color: Color::linear_rgba(0., 0., 0., 0.),
            alpha_mode: AlphaMode::Mask(0.5),
            ..default()
        }),
        material_fog: materials.add(StandardMaterial {
            base_color: Color::linear_rgb(0.01, 0.01, 0.01),
            specular_tint: Color::linear_rgb(0., 0., 0.),
            reflectance: 0.0,
            perceptual_roughness: 1.0,

            ..default()
        }),
        material_outlet: materials.add(StandardMaterial {
            base_color: Color::linear_rgb(0.0, 0.05, 0.1),
            perceptual_roughness: 1.0,
            base_color_texture: Some(asset_server.load("outlet.png")),
            alpha_mode: AlphaMode::Mask(0.5),
            ..default()
        }),
        material_icon_e: materials.add(StandardMaterial {
            base_color_texture: Some(image_e.clone()),
            alpha_mode: AlphaMode::Mask(0.5),
            ..default()
        }),
        material_icon_low_power: materials.add(StandardMaterial {
            base_color_texture: Some(image_low_power.clone()),
            alpha_mode: AlphaMode::Mask(0.5),
            ..default()
        }),

        image_e,

        image_fwd: asset_server.load("email_fwd.png"),

        scene_computer: asset_server.load("computer_console.glb#Scene0"),
        scene_ruby: asset_server.load("ruby.glb#Scene0"),

        material_tutorial_move: materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load("tutorial_move.png")),
            perceptual_roughness: 1.0,
            ..default()
        }),

        material_tutorial_interact: materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load("tutorial_interact.png")),
            perceptual_roughness: 1.0,
            ..default()
        }),

        material_tutorial_zip: materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load("tutorial_zip.png")),
            perceptual_roughness: 1.0,
            ..default()
        }),

        material_you_win: materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load("win_message.png")),
            perceptual_roughness: 1.0,
            ..default()
        }),

        material_email: materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load("email.png")),
            alpha_mode: AlphaMode::Mask(0.5),
            cull_mode: None,
            perceptual_roughness: 1.0,
            double_sided: true,
            ..default()
        }),
    });
}
