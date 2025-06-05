use std::sync::Mutex;

use avian3d::prelude::*;
use bevy::{
    color::color_difference::EuclideanDistance, image::ImageLoaderSettings,
    platform::collections::HashSet, prelude::*,
};

use crate::{
    common::Common,
    door::Door,
    draggable::Draggable,
    evil_robot::EvilRobot,
    laser::Laser,
    mainframe::Mainframe,
    player::Player,
    spawn_point::SpawnPoint,
    well::Well,
    zipline::Zipline,
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
        spawn: Box<dyn FnMut(&mut Commands, &SpawnInfo) + 'a>,
        skip_floor: bool,
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

    let zipline_positions: Mutex<Vec<IVec2>> = Mutex::new(Vec::new());

    let mut color_spawners: Vec<LevelSpawner> = vec![
        // White == Floor
        LevelSpawner {
            color: Color::linear_rgb(1., 1., 1.),
            spawn: Box::new(|_commands, _info| {
                // Nothing additional.
            }),
            skip_floor: false,
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
            skip_floor: false,
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
            skip_floor: false,
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
            skip_floor: false,
        },
        // Dark Grey == Well
        LevelSpawner {
            color: Color::linear_rgb(0.25, 0.25, 0.25),
            spawn: Box::new(|commands, info| {
                commands.spawn((
                    Mesh3d(common.mesh_sphere.clone()),
                    MeshMaterial3d(common.material_dark_gray.clone()),
                    Transform::from_translation(info.pos),
                    GlobalTransform::default(),
                    Well,
                ));
            }),
            skip_floor: true,
        },
        // Purple == Door
        LevelSpawner {
            color: Color::linear_rgb(0.5, 0.0, 1.0),
            spawn: Box::new(|commands, info| {
                // visual "wall" blocks above door
                commands.spawn((
                    Mesh3d(common.mesh_cube.clone()),
                    MeshMaterial3d(common.material_dark_gray.clone()),
                    Transform::from_translation(info.pos + Vec3::Y),
                    RigidBody::Static,
                    Collider::cuboid(1.0, 1.0, 1.0),
                    Door,
                ));

                // invisible blocker above that (like black wall)
                commands.spawn((
                    Mesh3d(common.mesh_cube.clone()),
                    MeshMaterial3d(common.material_invisible.clone()),
                    Transform::from_translation(info.pos + Vec3::Y * 2.0),
                    RigidBody::Static,
                    Collider::cuboid(1.0, 1.0, 1.0),
                ));
            }),
            skip_floor: false,
        },
        // Orange == Power Cell
        LevelSpawner {
            color: Color::linear_rgb(1., 0.5, 0.0),
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
            skip_floor: false,
        },
        // Pink == Laser Source
        LevelSpawner {
            color: Color::linear_rgb(1., 0.5, 0.5),
            spawn: Box::new(|commands, info| {
                // Wall
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

                for d in [IVec2::X, IVec2::Y, IVec2::NEG_X, IVec2::NEG_Y] {
                    let neighbor = info.grid + d;
                    let neighbor_color = image
                        .get_color_at(neighbor.x as u32, neighbor.y as u32)
                        .unwrap();
                    if neighbor_color.distance(&Color::linear_rgb(1., 1., 1.)) < 0.1 {
                        // Spawn laser in this direction
                        commands.spawn((
                            Mesh3d(common.mesh_cube.clone()),
                            MeshMaterial3d(common.material_red.clone()),
                            Transform::from_translation(
                                info.pos + Vec3::Y + Vec3::new(d.x as f32, 0.0, d.y as f32) * 0.5,
                            )
                            .with_scale(Vec3::splat(0.5)),
                            Laser {
                                direction: Vec3::new(d.x as f32, 0.0, d.y as f32),
                                beam: None,
                            },
                        ));
                    }
                }
            }),
            skip_floor: false,
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
            skip_floor: false,
        },
        // Yellow == Save/Spawn Point
        LevelSpawner {
            color: Color::linear_rgb(1., 1., 0.),
            spawn: Box::new(|commands, info| {
                commands.spawn((
                    Mesh3d(common.mesh_cube.clone()),
                    MeshMaterial3d(common.material_invisible.clone()),
                    Transform::from_translation(info.pos + Vec3::new(0.0, 2.0, 0.0)),
                    Collider::cuboid(1.0, 1.0, 1.0),
                    SpawnPoint {},
                ));
            }),
            skip_floor: false,
        },
        // Magenta == Zipline
        LevelSpawner {
            color: Color::linear_rgb(1., 0., 1.),
            spawn: Box::new(|_commands, info| {
                zipline_positions.lock().unwrap().push(info.grid);
            }),
            skip_floor: false,
        },
        // Dark Magenta == Zipline without floor
        LevelSpawner {
            color: Color::linear_rgb(0.5, 0., 0.5),
            spawn: Box::new(|_commands, info| {
                zipline_positions.lock().unwrap().push(info.grid);
            }),
            skip_floor: true,
        },
    ];

    for x in 0..image.width() {
        for z in 0..image.height() {
            let color = image.get_color_at(x, z).expect("must be able to get color");

            let color_distance_scale = 10_000;

            let (color_distance, candidate) = color_spawners
                .iter_mut()
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

            if !candidate.skip_floor {
                spawn_floor(&mut commands, info.pos);
            }

            (candidate.spawn)(&mut commands, &info);
        }
    }

    std::mem::drop(color_spawners);

    // Spawn ziplines
    spawn_ziplines(
        shift,
        &mut commands,
        &common,
        &zipline_positions.lock().unwrap(),
    );
}

fn spawn_ziplines(
    shift: Vec3,
    commands: &mut Commands,
    common: &Common,
    zipline_positions: &[IVec2],
) {
    let zipline_positions: HashSet<IVec2> = zipline_positions.iter().copied().collect();

    fn neighbors8(p: IVec2) -> Vec<IVec2> {
        let mut out: Vec<IVec2> = Vec::with_capacity(8);
        for dx in -1..=1 {
            for dy in -1..=1 {
                if (dx, dy) != (0, 0) {
                    out.push(p + IVec2::new(dx, dy));
                }
            }
        }
        out
    }

    // Group the ziplines into contiguous loops or lines.
    let mut visited: HashSet<IVec2> = HashSet::new();
    for &p in zipline_positions.iter() {
        if visited.contains(&p) {
            continue;
        }

        if neighbors8(p)
            .into_iter()
            .filter(|neighbor| zipline_positions.contains(neighbor))
            .count()
            != 1
        {
            // Only visit from the ends of a line, so that the region is built in-order.
            continue;
        }

        visited.insert(p);
        let mut region = vec![p];
        let mut i = 0;
        while i < region.len() {
            let q = region[i];
            for neighbor in neighbors8(q) {
                if !visited.contains(&neighbor) && zipline_positions.contains(&neighbor) {
                    visited.insert(neighbor);
                    region.push(neighbor);
                }
            }
            i += 1;
        }

        spawn_zipline(shift, commands, common, &region);
    }
}

/// Spawn a single zipline, in order.
fn spawn_zipline(
    shift: Vec3,
    commands: &mut Commands,
    common: &Common,
    zipline_positions: &[IVec2],
) {
    let mut nodes: Vec<Vec3> = Vec::new();
    for i in 0..zipline_positions.len() - 1 {
        let a = zipline_positions[i];
        let b = zipline_positions[i + 1];

        let height = 0.5;

        let end_a = a.as_vec2().extend(height).xzy() + shift;
        let end_b = b.as_vec2().extend(height).xzy() + shift;

        nodes.push((end_a + end_b) / 2.);

        commands.spawn((
            MeshMaterial3d(common.material_yellow.clone()),
            Mesh3d(common.mesh_cube.clone()),
            Transform::from_translation((end_a + end_b) / 2.)
                .with_scale(Vec3::new(0.4, 0.4, end_a.distance(end_b) + 0.4))
                .looking_at(end_a, Vec3::Y),
        ));
    }

    commands.spawn((
        Transform::from_translation(nodes[0]),
        Zipline {
            nodes,
            active: None,
            closest_index: 0,
        },
    ));
}
