use avian3d::prelude::*;
use bevy::{color::color_difference::EuclideanDistance, image::ImageLoaderSettings, prelude::*};

use crate::{
    common::Common, draggable::Draggable, evil_robot::EvilRobot, mainframe::Mainframe,
    player::Player,
};

pub struct LevelPlugin;

#[derive(Resource)]
pub struct Levels {
    map1: Handle<Image>,
}

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_levels_system)
            .add_systems(Update, load_level_system);
    }
}

fn setup_levels_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(Levels {
        map1: asset_server.load_with_settings("map1.png", |settings: &mut ImageLoaderSettings| {
            settings.is_srgb = false; // It's linear
        }),
    });
}

fn load_level_system(
    mut commands: Commands,
    levels: Res<Levels>,
    images: Res<Assets<Image>>,
    mut already_loaded: Local<bool>,

    common: Res<Common>,
) {
    if *already_loaded {
        return;
    }
    let Some(image) = images.get(&levels.map1) else {
        return;
    };
    *already_loaded = true;
    println!("loading {} x {} level", image.width(), image.height());

    #[allow(unused)]
    struct SpawnInfo {
        pos: Vec3,
        grid: IVec2,
    }

    let shift = Vec3::new(
        image.width() as f32 * -0.5,
        0.0,
        image.height() as f32 * -0.5,
    );

    struct LevelSpawner<'a> {
        color: Color,
        spawn: Box<dyn Fn(&mut Commands, &SpawnInfo) + 'a>,
    }

    let spawn_cube = |commands: &mut Commands, p: Vec3, material: Handle<StandardMaterial>| {
        commands.spawn((
            Mesh3d(common.mesh_cube.clone()),
            MeshMaterial3d(material),
            Transform::from_translation(p),
            RigidBody::Static,
            Collider::cuboid(1., 1., 1.),
        ));
    };

    let spawn_floor = |commands: &mut Commands, p: Vec3| {
        spawn_cube(commands, p, common.material_gray.clone());
    };

    let color_spawners: Vec<LevelSpawner> = vec![
        // White == Floor
        LevelSpawner {
            color: Color::linear_rgb(1., 1., 1.),
            spawn: Box::new(|_commands, _info| {
                // Nothing additional.
            }),
        },
        // Black == Wall
        LevelSpawner {
            color: Color::linear_rgb(0., 0., 0.),
            spawn: Box::new(|commands, info| {
                spawn_cube(
                    commands,
                    info.pos + Vec3::Y,
                    common.material_dark_gray.clone(),
                );
                spawn_cube(
                    commands,
                    info.pos + Vec3::Y * 2.,
                    common.material_invisible.clone(),
                );
            }),
        },
        // Green == Compute
        LevelSpawner {
            color: Color::linear_rgb(0., 1., 0.),
            spawn: Box::new(|commands, info| {
                commands.spawn((
                    Mesh3d(common.mesh_cube.clone()),
                    MeshMaterial3d(common.material_gray.clone()),
                    Transform::from_translation(info.pos + Vec3::Y),
                    RigidBody::Static,
                    Collider::cuboid(1., 1., 1.),
                    Mainframe { active: false },
                ));
            }),
        },
        // Blue == Robot
        LevelSpawner {
            color: Color::linear_rgb(0., 0., 1.),
            spawn: Box::new(|commands, info| {
                commands.spawn((
                    Mesh3d(common.mesh_sphere.clone()),
                    MeshMaterial3d(common.material_beepboop.clone()),
                    Transform::from_translation(info.pos + Vec3::Y),
                    RigidBody::Dynamic,
                    Collider::cuboid(1., 1., 1.),
                    EvilRobot {},
                ));
            }),
        },
        // Orange == Power Cell
        LevelSpawner {
            color: Color::linear_rgb(1., 0.5, 0.),
            spawn: Box::new(|commands, info| {
                commands.spawn((
                    Mesh3d(common.mesh_cube.clone()),
                    MeshMaterial3d(common.material_red.clone()),
                    Transform::from_translation(info.pos + Vec3::Y).with_scale(Vec3::splat(0.8)),
                    RigidBody::Dynamic,
                    Collider::cuboid(1.0, 1.0, 1.0),
                    Draggable::default(),
                ));
            }),
        },
        // Red == Player
        LevelSpawner {
            color: Color::linear_rgb(1., 0., 0.),
            spawn: Box::new(|commands, info| {
                commands.spawn((
                    Mesh3d(common.mesh_sphere.clone()),
                    MeshMaterial3d(common.material_gray.clone()),
                    Transform::from_translation(info.pos + Vec3::new(0.0, 2., 0.)),
                    RigidBody::Dynamic,
                    Collider::sphere(0.45),
                    Player {},
                ));
            }),
        },
        // Yellow == Save/Spawn Point
        LevelSpawner {
            color: Color::linear_rgb(1., 1., 0.),
            spawn: Box::new(|_commands, _info| {
                // TODO: Spawn me
            }),
        },
    ];

    for x in 0..image.width() {
        for z in 0..image.height() {
            let color = image.get_color_at(x, z).expect("must be able to get color");

            let color_distance_scale = 10_000;

            let (color_distance, candidate) = color_spawners
                .iter()
                .map(|candidate| (candidate.color.distance(&color), candidate))
                .min_by_key(|a| (a.0 * color_distance_scale as f32) as i64)
                .unwrap();

            if color_distance > 0.1 {
                eprintln!("unknown color {:?}", color);
                continue;
            }

            let info = SpawnInfo {
                pos: Vec3::new(x as f32, 0.0, z as f32) + shift,
                grid: IVec2::new(x as i32, z as i32),
            };

            spawn_floor(&mut commands, info.pos);

            (candidate.spawn)(&mut commands, &info);
        }
    }
}
