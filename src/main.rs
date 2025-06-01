pub mod common;
pub mod draggable;
pub mod interactible;
pub mod mainframe;
pub mod player;
use avian3d::prelude::*;

use bevy::{
    core_pipeline::{
        bloom::{Bloom, BloomPrefilter},
        tonemapping::Tonemapping,
    },
    prelude::*,
    render::view::{ColorGrading, ColorGradingGlobal},
};
use common::{Common, CommonPlugin, setup_common};
use draggable::{Draggable, DraggablePlugin};
use interactible::InteractiblePlugin;
use mainframe::{Mainframe, MainframePlugin};
use player::{Player, PlayerCamera, PlayerPlugin};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            PhysicsPlugins::default(),
            PlayerPlugin,
            InteractiblePlugin,
            MainframePlugin,
            DraggablePlugin,
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

    for (x, z) in [(-3, -3), (5, 2), (8, 8)] {
        commands.spawn((
            Mesh3d(common.mesh_cube.clone()),
            MeshMaterial3d(common.material_gray.clone()),
            Transform::from_translation(Vec3::new(x as f32, 1.2, z as f32)),
            RigidBody::Static,
            Collider::cuboid(1., 1., 1.),
            Mainframe { active: false },
        ));

        commands.spawn((
            Mesh3d(common.mesh_cube.clone()),
            MeshMaterial3d(common.material_red.clone()),
            Transform::from_translation(
                Vec3::new(x as f32, 1.2, z as f32) + Vec3::new(-3., 2., 0.),
            )
            .with_scale(Vec3::splat(0.8)),
            RigidBody::Dynamic,
            Collider::cuboid(0.8, 0.8, 0.8),
            Draggable::default(),
        ));
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
        Camera {
            hdr: true,

            ..default()
        },
        Tonemapping::SomewhatBoringDisplayTransform,
        ColorGrading {
            global: ColorGradingGlobal {
                post_saturation: 1.25,
                ..default()
            },
            ..default()
        },
        Bloom {
            prefilter: BloomPrefilter {
                threshold: 0.6,
                threshold_softness: 0.2,
            },
            ..Bloom::NATURAL
        },
        Transform::from_xyz(0.0, 17., 14.0).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
        PlayerCamera,
    ));

    commands.spawn((
        Mesh3d(common.mesh_sphere.clone()),
        MeshMaterial3d(common.material_gray.clone()),
        Transform::from_translation(Vec3::new(0.0, 7., 0.)),
        RigidBody::Dynamic,
        Collider::sphere(0.5),
        Player {},
    ));
}
