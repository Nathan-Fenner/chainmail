pub mod chain;
pub mod common;
pub mod door;
pub mod draggable;
pub mod electricity;
pub mod evil_robot;
pub mod fog;
pub mod interactible;
pub mod intro;
pub mod laser;
pub mod level;
pub mod mainframe;
pub mod player;
pub mod ruby;
pub mod spawn_point;
pub mod well;
pub mod zipline;

use avian3d::prelude::*;

use bevy::{
    core_pipeline::{
        bloom::{Bloom, BloomPrefilter},
        tonemapping::Tonemapping,
    },
    prelude::*,
    render::view::{ColorGrading, ColorGradingGlobal},
};
use common::{CommonPlugin, setup_common};
use door::DoorPlugin;
use draggable::DraggablePlugin;
use evil_robot::EvilRobotPlugin;
use interactible::InteractiblePlugin;
use mainframe::MainframePlugin;
use player::{PlayerCamera, PlayerPlugin};
use spawn_point::SpawnPointPlugin;
use well::WellPlugin;

use crate::{
    chain::ChainPlugin, electricity::ElectricityPlugin, fog::FogPlugin, intro::IntroPlugin,
    laser::LaserPlugin, level::LevelPlugin, ruby::RubyPlugin, zipline::ZiplinePlugin,
};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            PhysicsPlugins::default(),
            PlayerPlugin,
            InteractiblePlugin,
            MainframePlugin,
            LevelPlugin,
            DraggablePlugin,
            EvilRobotPlugin,
            ZiplinePlugin,
            SpawnPointPlugin,
            CommonPlugin,
            LaserPlugin,
            WellPlugin,
            DoorPlugin,
            FogPlugin,
        ))
        .add_plugins((IntroPlugin, ChainPlugin, RubyPlugin, ElectricityPlugin))
        .add_systems(Startup, setup.after(setup_common))
        .run();
}

fn setup(mut commands: Commands) {
    // overhead lighting
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            shadow_depth_bias: 0.2,
            ..default()
        },
        Transform::from_xyz(8.0, 16.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // camera
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
}
