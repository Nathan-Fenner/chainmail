pub mod common;
pub mod player;
use avian3d::prelude::*;

use bevy::prelude::*;
use common::{Common, CommonPlugin, setup_common};
use player::{Player, PlayerCamera, PlayerPlugin};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            PhysicsPlugins::default(),
            PlayerPlugin,
        ))
        .add_plugins(CommonPlugin)
        .add_systems(Startup, (setup_common, setup).chain())
        .run();
}

fn setup(mut commands: Commands, common: Res<Common>) {
    for x in -10..=10 {
        for z in -10..=10 {
            commands.spawn((
                Mesh3d(common.mesh_cube.clone()),
                MeshMaterial3d(common.material_gray.clone()),
                Transform::from_translation(Vec3::new(x as f32, 0.0, z as f32)),
                RigidBody::Static,
                Collider::cuboid(1., 1., 1.),
            ));
        }
    }

    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            shadow_depth_bias: 0.2,
            ..default()
        },
        Transform::from_xyz(8.0, 16.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 17., 14.0).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
        PlayerCamera,
    ));

    commands.spawn((
        Mesh3d(common.mesh_sphere.clone()),
        MeshMaterial3d(common.material_gray.clone()),
        Transform::from_translation(Vec3::new(0.0, 7., 0.)),
        RigidBody::Dynamic,
        Collider::sphere(1.),
        Player {},
    ));
}
