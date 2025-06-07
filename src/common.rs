use bevy::prelude::*;

#[derive(Resource)]
pub struct Common {
    pub mesh_cube: Handle<Mesh>,
    pub mesh_sphere: Handle<Mesh>,
    pub mesh_cylinder: Handle<Mesh>,
    pub mesh_plane: Handle<Mesh>,
    pub mesh_small_sphere: Handle<Mesh>,
    pub material_gray: Handle<StandardMaterial>,
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

    pub image_e: Handle<Image>,

    pub material_icon_e: Handle<StandardMaterial>,
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
    commands.insert_resource(Common {
        mesh_cube: meshes.add(Cuboid::default()),
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
        image_e,
    });
}
