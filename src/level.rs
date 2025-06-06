use std::sync::Mutex;

use avian3d::{parry::utils::hashmap::HashMap, prelude::*};
use bevy::{
    color::color_difference::EuclideanDistance, image::ImageLoaderSettings,
    platform::collections::HashSet, prelude::*,
};

use crate::{
    common::Common, door::Door, draggable::Draggable, evil_robot::EvilRobot, laser::Laser,
    mainframe::Mainframe, player::Player, spawn_point::SpawnPoint, well::Well, zipline::Zipline,
};

pub struct LevelPlugin;

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub struct LevelName {
    level_name: String,
}

impl LevelName {
    fn from_string(level_name: String) -> Self {
        Self { level_name }
    }
}

impl std::fmt::Display for LevelName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.level_name)
    }
}

#[derive(Resource)]
pub struct Levels {
    levels: HashMap<LevelName, Handle<Image>>, // map1: Handle<Image>,
}

#[derive(Component, Clone, Eq, PartialEq, Debug, Hash)]
pub struct LevelTag {
    pub level: LevelName,
}

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_levels_system)
            .add_systems(Update, load_level_system);
    }
}

fn setup_levels_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    let map_names = ["map1.png", "map2.png"];

    commands.insert_resource(Levels {
        levels: map_names
            .iter()
            .map(|map_name| {
                (
                    LevelName::from_string(map_name.to_string()),
                    asset_server.load_with_settings(
                        *map_name,
                        |settings: &mut ImageLoaderSettings| {
                            settings.is_srgb = false; // It's linear
                        },
                    ),
                )
            })
            .collect(),
    });
}

#[allow(unused)]
struct SpawnInfo {
    pos: Vec3,
    grid: IVec2,
}

#[derive(Component)]
struct Hallway {
    pattern: HallwayPattern,
    room1: LevelName,
    room2: LevelName,
}

fn load_level_system(
    mut commands: Commands,
    levels: Res<Levels>,
    image_assets: Res<Assets<Image>>,
    mut active_levels: Local<HashMap<LevelName, Vec3>>,
    mut has_loaded_player: Local<bool>,

    common: Res<Common>,

    level_items: Query<(Entity, &Transform, &LevelTag)>,
    player: Query<&Transform, With<Player>>,
    hallways: Query<(&Transform, &Hallway, &LevelTag)>,

    mut hallway_junctions: Local<HashMap<LevelName, Vec<HallwayJunction>>>,
) {
    // Check that all levels are loaded
    for level in levels.levels.values() {
        if !image_assets.contains(level) {
            // This level has not yet loaded.
            return;
        }
    }

    if hallway_junctions.is_empty() {
        *hallway_junctions = levels
            .levels
            .iter()
            .map(|(level, image)| {
                (
                    level.clone(),
                    get_hallway_junctions(image_assets.get(image).unwrap()),
                )
            })
            .collect();
    }

    // If the player is in a hallway, load both levels.
    // If there is no player, load the first level.

    if !*has_loaded_player {
        let first_level = LevelName::from_string("map1.png".to_string());
        load_level(
            Vec3::ZERO,
            LevelTag {
                level: first_level.clone(),
            },
            &mut commands,
            &common,
            image_assets.get(&levels.levels[&first_level]).unwrap(),
            true,
        );
        *has_loaded_player = true;
        active_levels.insert(first_level, Vec3::ZERO);
        return;
    }

    // Figure out which room the player is in.
    let Ok(player) = player.single() else {
        return;
    };

    // If the player is in a hallway, this is a special case.

    let mut is_in_hall = false;
    for (hallway_transform, hallway, hallway_level) in hallways.iter() {
        if hallway_transform.translation.distance(player.translation) < 1.0 {
            is_in_hall = true;
            // Player is in the hallway!
            for level_to_load in [&hallway.room1, &hallway.room2] {
                if !active_levels.contains_key(level_to_load) {
                    let old_level_junctions = &hallway_junctions[&hallway_level.level];
                    let new_level_image = image_assets.get(&levels.levels[level_to_load]).unwrap();
                    let new_level_junctions = &hallway_junctions[level_to_load];

                    let Some(old_hallway) = old_level_junctions
                        .iter()
                        .find(|h| h.pattern == hallway.pattern)
                    else {
                        eprintln!(
                            "failed to load new level {} from level {} - the old hallway pattern not found",
                            level_to_load, hallway_level.level,
                        );
                        continue;
                    };

                    let Some(new_hallway) = new_level_junctions
                        .iter()
                        .find(|h| h.pattern == hallway.pattern)
                    else {
                        eprintln!(
                            "failed to load new level {} from level {} - the new hallway pattern not found",
                            level_to_load, hallway_level.level,
                        );
                        continue;
                    };

                    let old_shift = active_levels[&hallway_level.level];
                    // let old_hall_at = old_shift + Vec3::new(old_hallway.center.x, 0.0, old_hallway.center.y)
                    // let new_hall_at = new_shift + Vec3::new(new_hallway.center.x, 0.0, new_hallway.center.y)
                    let new_shift = old_shift
                        + Vec3::new(old_hallway.center.x, 0.0, old_hallway.center.y)
                        - Vec3::new(new_hallway.center.x, 0.0, new_hallway.center.y);

                    load_level(
                        new_shift,
                        LevelTag {
                            level: level_to_load.clone(),
                        },
                        &mut commands,
                        &common,
                        new_level_image,
                        false,
                    );
                    active_levels.insert(level_to_load.clone(), new_shift);
                }
            }
        }
    }

    if !is_in_hall && active_levels.len() >= 2 {
        // Despawn all of the other levels.
        let closest_level = level_items.iter().min_by_key(|(_, t, _level)| {
            (t.translation.distance(player.translation) * 100.) as i64
        });

        if let Some((_, _, closest_level)) = closest_level {
            let mut must_keep: HashSet<&LevelName> = HashSet::new();
            for (_, t, level) in level_items.iter() {
                if t.translation.distance(player.translation) < 2.5 {
                    must_keep.insert(&level.level);
                }
            }

            for (entity, _, level) in level_items.iter() {
                if must_keep.contains(&level.level) {
                    continue;
                }
                if level != closest_level {
                    commands.entity(entity).despawn();
                }
            }
            active_levels.retain(|key, _| key == &closest_level.level || must_keep.contains(key));
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
struct HallwayPattern(u32);

#[derive(Debug)]
struct HallwayJunction {
    pattern: HallwayPattern,
    center: Vec2,
    grids: Vec<IVec2>,
}

fn get_hallway_junctions(image: &Image) -> Vec<HallwayJunction> {
    fn is_hallway_color(c: &Color) -> bool {
        c.distance(&Color::linear_rgb(0.5, 0.5, 0.5)) < 0.1
            || c.distance(&Color::linear_rgb(0.75, 0.75, 0.75)) < 0.1
    }
    let mut visited: HashSet<IVec2> = HashSet::new();
    let mut patterns: Vec<HallwayJunction> = Vec::new();

    for x in 0..image.width() as i32 {
        for y in 0..image.height() as i32 {
            let p = IVec2::new(x, y);

            if visited.contains(&p) {
                continue;
            }

            let color = image.get_color_at(x as u32, y as u32).unwrap();
            if !is_hallway_color(&color) {
                continue;
            }

            visited.insert(p);
            let mut region: HashSet<IVec2> = HashSet::new();
            region.insert(p);
            let mut stack = vec![p];
            while let Some(curr) = stack.pop() {
                for d in [IVec2::X, IVec2::NEG_X, IVec2::Y, IVec2::NEG_Y] {
                    let neighbor = curr + d;
                    if visited.contains(&neighbor) {
                        continue;
                    }
                    let neighbor_color = image
                        .get_color_at(neighbor.x as u32, neighbor.y as u32)
                        .unwrap();
                    if !is_hallway_color(&neighbor_color) {
                        continue;
                    }

                    visited.insert(neighbor);
                    region.insert(neighbor);
                    stack.push(neighbor);
                }
            }

            let mut region_pattern = region.iter().copied().collect::<Vec<IVec2>>();
            region_pattern.sort_by_key(|p| (p.x, p.y));
            let region_pattern: HallwayPattern = HallwayPattern(
                region_pattern
                    .iter()
                    .map(|p| {
                        if image
                            .get_color_at(p.x as u32, p.y as u32)
                            .unwrap()
                            .to_linear()
                            .red
                            < 0.62
                        {
                            1
                        } else {
                            0
                        }
                    })
                    .fold(0, |a, b| a * 2 + b),
            );

            patterns.push(HallwayJunction {
                pattern: region_pattern,
                center: region.iter().map(|p| p.as_vec2()).sum::<Vec2>() / region.len() as f32,
                grids: region.iter().copied().collect(),
            })
        }
    }

    patterns
}

fn load_level(
    shift: Vec3,
    level_tag: LevelTag,
    commands: &mut Commands,
    common: &Common,
    image: &Image,
    should_spawn_player: bool,
) {
    struct LevelSpawner<'a> {
        color: Color,
        spawn: Box<dyn FnMut(&mut Commands, &SpawnInfo) + 'a>,
        skip_floor: bool,
    }

    let spawn_cube = |commands: &mut Commands, p: Vec3, material: Handle<StandardMaterial>| {
        commands.spawn((
            level_tag.clone(),
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
        // Light Grey == Connecting Hallway Floor
        LevelSpawner {
            color: Color::linear_rgb(0.75, 0.75, 0.75),
            spawn: Box::new(|_commands, _info| {
                // Spawned later
            }),
            skip_floor: false,
        },
        // Medium Grey == Connecting Hallway Floor
        LevelSpawner {
            color: Color::linear_rgb(0.5, 0.5, 0.5),
            spawn: Box::new(|_commands, _info| {
                // Spawned later
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
                    level_tag.clone(),
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
        // Light Blue == Outside
        LevelSpawner {
            color: Color::linear_rgb(0.5, 0.5, 0.835),
            spawn: Box::new(|_commands, _info| {
                // Nothing at all
            }),
            skip_floor: true,
        },
        // Blue == Robot
        LevelSpawner {
            color: Color::linear_rgb(0., 0., 1.),
            spawn: Box::new(|commands, info| {
                commands.spawn((
                    level_tag.clone(),
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
                    level_tag.clone(),
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
                    level_tag.clone(),
                    Mesh3d(common.mesh_cube.clone()),
                    MeshMaterial3d(common.material_dark_gray.clone()),
                    Transform::from_translation(info.pos + Vec3::Y),
                    RigidBody::Static,
                    Collider::cuboid(1.0, 1.0, 1.0),
                    Door,
                ));

                // invisible blocker above that (like black wall)
                commands.spawn((
                    level_tag.clone(),
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
                    level_tag.clone(),
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
                            level_tag.clone(),
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
                if !should_spawn_player {
                    return;
                }
                commands.spawn((
                    // No level tag on the player
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
                    level_tag.clone(),
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
                spawn_floor(commands, info.pos);
            }

            (candidate.spawn)(commands, &info);
        }
    }

    std::mem::drop(color_spawners);

    // Spawn ziplines
    spawn_ziplines(
        shift,
        &level_tag,
        commands,
        common,
        &zipline_positions.lock().unwrap(),
    );

    for hallway_pattern in get_hallway_junctions(image) {
        for p in hallway_pattern.grids.iter() {
            commands.spawn((
                level_tag.clone(),
                Mesh3d(common.mesh_cube.clone()),
                MeshMaterial3d(common.material_invisible.clone()),
                Transform::from_translation(shift + Vec3::new(p.x as f32, 1.0, p.y as f32))
                    .with_scale(Vec3::splat(0.7)),
                Hallway {
                    pattern: hallway_pattern.pattern,
                    room1: LevelName::from_string("map1.png".to_string()),
                    room2: LevelName::from_string("map2.png".to_string()),
                },
            ));
        }
    }
}

fn spawn_ziplines(
    shift: Vec3,
    level_tag: &LevelTag,
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

        spawn_zipline(shift, level_tag, commands, common, &region);
    }
}

/// Spawn a single zipline, in order.
fn spawn_zipline(
    shift: Vec3,
    level_tag: &LevelTag,
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
            level_tag.clone(),
            MeshMaterial3d(common.material_yellow.clone()),
            Mesh3d(common.mesh_cube.clone()),
            Transform::from_translation((end_a + end_b) / 2.)
                .with_scale(Vec3::new(0.4, 0.4, end_a.distance(end_b) + 0.4))
                .looking_at(end_a, Vec3::Y),
        ));
    }

    commands.spawn((
        level_tag.clone(),
        Transform::from_translation(nodes[0]),
        Zipline {
            nodes,
            active: None,
            closest_index: 0,
        },
    ));
}
