use bevy::prelude::*;

#[derive(Resource)]
pub struct Common {
    pub mesh_cube: Handle<Mesh>,
    pub mesh_sphere: Handle<Mesh>,
    pub material_gray: Handle<StandardMaterial>,
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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(Common {
        mesh_cube: meshes.add(Cuboid::default()),
        mesh_sphere: meshes.add(Sphere::default()),
        material_gray: materials.add(StandardMaterial {
            base_color: Color::linear_rgb(0.4, 0.5, 0.6),
            perceptual_roughness: 1.0,
            ..default()
        }),
    });
}
