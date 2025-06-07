use std::sync::Mutex;

use avian3d::{parry::utils::hashmap::HashMap, prelude::*};
use bevy::{
    color::color_difference::EuclideanDistance, image::ImageLoaderSettings,
    platform::collections::HashSet, prelude::*,
};

use crate::{
    chain::ChainLink,
    common::Common,
    door::Door,
    draggable::Draggable,
    electricity::{Outlet, Plug},
    evil_robot::EvilRobot,
    fog::DoesNotClearFog,
    laser::Laser,
    mainframe::Mainframe,
    player::Player,
    spawn_point::SpawnPoint,
    well::Well,
    zipline::Zipline,
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
    let map_names = ["map1.png", "rails_map.png"];

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
    rooms: Vec<LevelName>,
}

struct Resetting {
    time_left: f32,
    locus: Vec3,
}

fn load_level_system(
    time: Res<Time>,
    mut commands: Commands,
    levels: Res<Levels>,
    image_assets: Res<Assets<Image>>,
    mut active_levels: Local<HashMap<LevelName, Vec3>>,
    // These levels are being reset, but may be restored if needed.
    mut resetting_levels: Local<HashMap<LevelName, Resetting>>,
    mut has_loaded_player: Local<bool>,

    common: Res<Common>,

    level_items: Query<(Entity, &Transform, &LevelTag)>,
    player: Query<&Transform, With<Player>>,
    hallways: Query<(&Transform, &Hallway, &LevelTag)>,

    mut hallway_junctions: Local<HashMap<LevelName, Vec<HallwayJunction>>>,
    mut junction_to_levels: Local<HashMap<HallwayPattern, Vec<LevelName>>>,
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

        for (level, junctions) in hallway_junctions.iter() {
            for junction in junctions.iter() {
                junction_to_levels
                    .entry(junction.pattern)
                    .or_default()
                    .push(level.clone());
            }
        }
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
            &junction_to_levels,
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
            for level_to_load in hallway.rooms.iter() {
                if resetting_levels.contains_key(level_to_load) {
                    // Restore the level instead of allowing it to despawn.
                    active_levels.insert(
                        level_to_load.clone(),
                        resetting_levels.remove(level_to_load).unwrap().locus,
                    );
                    for (item_entity, _, level) in level_items.iter() {
                        if level.level == *level_to_load {
                            commands.entity(item_entity).remove::<DoesNotClearFog>();
                        }
                    }
                }

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
                        &junction_to_levels,
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

            let current_active_levels = std::mem::take(&mut *active_levels);
            for (current_key, current_state) in current_active_levels {
                if current_key == closest_level.level || must_keep.contains(&current_key) {
                    // Retain this level.
                    active_levels.insert(current_key, current_state);
                } else {
                    // This level must be marked for deletion.
                    for (entity, _, level) in level_items.iter() {
                        if level.level == current_key {
                            commands.entity(entity).insert(DoesNotClearFog);
                        }
                    }
                    resetting_levels.insert(
                        current_key,
                        Resetting {
                            time_left: 4.0,
                            locus: current_state,
                        },
                    );
                }
            }
        }
    }

    resetting_levels.retain(|level, resetting| {
        let will_despawn = resetting.time_left < 0.0;
        resetting.time_left -= time.delta_secs();
        if will_despawn {
            for (entity, _, entity_level) in level_items.iter() {
                if entity_level.level == *level {
                    commands.entity(entity).despawn();
                }
            }
            // Remove the level from this list
            false
        } else {
            // Keep the level alive for now
            true
        }
    });
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
            || c.distance(&Color::linear_rgb(0.625, 0.625, 0.625)) < 0.1
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
                        let color = image
                            .get_color_at(p.x as u32, p.y as u32)
                            .unwrap()
                            .to_linear()
                            .red;
                        if color < 0.56 {
                            1
                        } else if color < 0.688 {
                            2
                        } else {
                            3
                        }
                    })
                    .fold(0, |a, b| a * 4 + b),
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
    junction_to_levels: &HashMap<HallwayPattern, Vec<LevelName>>,
) {
    struct LevelSpawner<'a> {
        color: Color,
        spawn: Box<dyn FnMut(&mut Commands, &SpawnInfo) + 'a>,
        skip_floor: bool,
        lift_entity: bool,
        lift_floor: bool,
    }
    impl<'a> LevelSpawner<'a> {
        fn new(color: Color, spawn: impl FnMut(&mut Commands, &SpawnInfo) + 'a) -> Self {
            Self {
                color,
                spawn: Box::new(spawn),
                skip_floor: false,
                lift_entity: false,
                lift_floor: false,
            }
        }

        fn skip_floor(self) -> Self {
            Self {
                skip_floor: true,
                ..self
            }
        }
        fn lift_floor(self) -> Self {
            Self {
                lift_entity: true,
                lift_floor: true,
                ..self
            }
        }
        fn lift_entity(self) -> Self {
            Self {
                lift_entity: true,
                lift_floor: false,
                ..self
            }
        }
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

    let zipline_positions: Mutex<HashMap<IVec2, Vec3>> = Mutex::new(HashMap::default());
    let chains: Mutex<HashMap<IVec2, Vec3>> = Mutex::new(HashMap::default());

    let is_raised = |p: IVec2| -> bool {
        let c = image.get_color_at(p.x as u32, p.y as u32).unwrap();

        c.distance(&Color::linear_rgb(186. / 255., 221. / 255., 1.)) < 0.1
            || c.distance(&Color::linear_rgb(0.5, 0.75, 1.0)) < 0.1
    };

    #[allow(clippy::eq_op)]
    let mut color_spawners: Vec<LevelSpawner> = vec![
        // White == Floor
        LevelSpawner::new(Color::linear_rgb(1., 1., 1.), |_commands, _info| {
            // Nothing additional.
        }),
        // Light Blue == Elevated Floor
        LevelSpawner::new(Color::linear_rgb(0.5, 0.75, 1.0), |commands, info| {
            spawn_cube(commands, info.pos + Vec3::Y, common.material_gray.clone());
        }),
        // Lighter Blue == Ramp
        LevelSpawner::new(
            Color::linear_rgb(186. / 255., 221. / 255., 1.),
            |commands, info| {
                let facing = Vec3::X;

                commands.spawn((
                    level_tag.clone(),
                    Mesh3d(common.mesh_cube.clone()),
                    MeshMaterial3d(common.material_gray.clone()),
                    Transform::from_translation(info.pos - facing * 0.5 + Vec3::Y * 0.5)
                        .looking_to(facing + Vec3::Y, Vec3::Y)
                        .with_scale(Vec3::new(1.0, 2.0f32.sqrt(), 2.0f32.sqrt())),
                    RigidBody::Static,
                    Collider::cuboid(1., 1., 1.),
                ));
            },
        ),
        // Light Grey == Connecting Hallway Floor
        LevelSpawner::new(Color::linear_rgb(0.75, 0.75, 0.75), |_commands, _info| {
            // Spawned later
        }),
        // Light Medium Grey == Connecting Hallway Floor
        LevelSpawner::new(
            Color::linear_rgb(0.625, 0.625, 0.625),
            |_commands, _info| {
                // Spawned later
            },
        ),
        // Medium Grey == Connecting Hallway Floor
        LevelSpawner::new(Color::linear_rgb(0.5, 0.5, 0.5), |_commands, _info| {
            // Spawned later
        }),
        // Black == Wall
        LevelSpawner::new(Color::linear_rgb(0., 0., 0.), |commands, info| {
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
        // Green == Compute
        LevelSpawner::new(Color::linear_rgb(0., 1., 0.), |commands, info| {
            commands.spawn((
                level_tag.clone(),
                Mesh3d(common.mesh_cube.clone()),
                MeshMaterial3d(common.material_gray.clone()),
                Transform::from_translation(info.pos + Vec3::Y),
                RigidBody::Static,
                Collider::cuboid(1., 1., 1.),
                Mainframe { active: false },
            ));
        })
        .lift_floor(),
        // Light Blue == Outside
        LevelSpawner::new(Color::linear_rgb(0.5, 0.5, 0.835), |_commands, _info| {
            // Nothing at all
        })
        .skip_floor(),
        // Blue == Robot
        LevelSpawner::new(Color::linear_rgb(0., 0., 1.), |commands, info| {
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
        // Dark Grey == Well
        LevelSpawner::new(Color::linear_rgb(0.25, 0.25, 0.25), |commands, info| {
            commands.spawn((
                level_tag.clone(),
                Mesh3d(common.mesh_sphere.clone()),
                MeshMaterial3d(common.material_dark_gray.clone()),
                Transform::from_translation(info.pos),
                GlobalTransform::default(),
                Well,
            ));
        })
        .skip_floor(),
        // Purple == Door
        LevelSpawner::new(Color::linear_rgb(0.5, 0.0, 1.0), |commands, info| {
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
        })
        .lift_floor(),
        // Orange == Power Cell
        LevelSpawner::new(Color::linear_rgb(1., 0.5, 0.0), |commands, info| {
            commands.spawn((
                level_tag.clone(),
                Mesh3d(common.mesh_cube.clone()),
                MeshMaterial3d(common.material_orange.clone()),
                Transform::from_translation(info.pos + Vec3::Y).with_scale(Vec3::splat(0.8)),
                RigidBody::Dynamic,
                Collider::cuboid(1.0, 1.0, 1.0),
                Draggable::default(),
            ));
        })
        .lift_floor(),
        // Pink == Laser Source
        LevelSpawner::new(Color::linear_rgb(1., 0.5, 0.5), |commands, info| {
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
                        MeshMaterial3d(common.material_orange.clone()),
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
        })
        .lift_floor(),
        // Red == Player
        LevelSpawner::new(Color::linear_rgb(1., 0., 0.), |commands, info| {
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
                GravityScale(2.),
            ));
        }),
        // Yellow == Save/Spawn Point
        LevelSpawner::new(Color::linear_rgb(1., 1., 0.), |commands, info| {
            commands.spawn((
                level_tag.clone(),
                Mesh3d(common.mesh_cube.clone()),
                MeshMaterial3d(common.material_invisible.clone()),
                Transform::from_translation(info.pos + Vec3::new(0.0, 2.0, 0.0)),
                Collider::cuboid(1.0, 1.0, 1.0),
                SpawnPoint {},
            ));
        })
        .lift_floor(),
        // Magenta == Zipline
        LevelSpawner::new(Color::linear_rgb(1., 0., 1.), |_commands, info| {
            zipline_positions
                .lock()
                .unwrap()
                .insert(info.grid, info.pos + Vec3::Y * 0.5);
        })
        .lift_floor(),
        // Dark Magenta == Zipline without floor
        LevelSpawner::new(Color::linear_rgb(0.5, 0., 0.5), |commands, info| {
            zipline_positions
                .lock()
                .unwrap()
                .insert(info.grid, info.pos + Vec3::Y * 0.5);

            commands.spawn((
                level_tag.clone(),
                Mesh3d(common.mesh_sphere.clone()),
                MeshMaterial3d(common.material_dark_gray.clone()),
                Transform::from_translation(info.pos),
                GlobalTransform::default(),
                Well,
            ));
        })
        .skip_floor()
        .lift_entity(),
        // Brown == Chain
        LevelSpawner::new(
            Color::linear_rgb(159.0 / 255.0, 113.0 / 255.0, 62.0 / 255.0),
            |_commands, info| {
                chains.lock().unwrap().insert(info.grid, info.pos + Vec3::Y);
            },
        ),
        // Pale Purple == Electricity Outlet
        LevelSpawner::new(
            Color::linear_rgb(158.0 / 255.0, 86.0 / 255.0, 158.0 / 255.0),
            |commands, info| {
                commands.spawn((
                    level_tag.clone(),
                    Mesh3d(common.mesh_cube.clone()),
                    MeshMaterial3d(common.material_outlet.clone()),
                    Transform::from_translation(info.pos + Vec3::Y * 0.5)
                        .with_scale(Vec3::new(0.8, 0.1, 0.8)),
                    RigidBody::Static,
                    Collider::cuboid(1.0, 1.0, 1.0),
                    Outlet { plug: None },
                ));
            },
        ),
        // Light Teal == Power Source
        LevelSpawner::new(
            Color::linear_rgb(128. / 255., 255. / 255., 221. / 255.),
            |commands, info| {
                commands.spawn((
                    level_tag.clone(),
                    Mesh3d(common.mesh_cylinder.clone()),
                    MeshMaterial3d(common.material_electricity.clone()),
                    Transform::from_translation(info.pos + Vec3::Y * 0.5)
                        .with_scale(Vec3::new(0.8, 1.5, 0.8)),
                    RigidBody::Static,
                    Collider::cylinder(0.5, 1.0),
                    Outlet { plug: None },
                ));
            },
        ),
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

            let mut info = SpawnInfo {
                pos: Vec3::new(x as f32, 0.0, z as f32) + shift,
                grid: IVec2::new(x as i32, z as i32),
            };

            let mut floor_height = 0.0;
            let mut entity_height = 0.0;
            if candidate.lift_entity || candidate.lift_floor {
                for dx in -1..=1 {
                    for dz in -1..=1 {
                        if is_raised(info.grid + IVec2::new(dx, dz)) {
                            entity_height = 1.0;
                            if dx == 0 || dz == 0 {
                                floor_height = 1.0;
                            }
                        }
                    }
                }
            }

            if !candidate.skip_floor {
                if candidate.lift_floor {
                    info.pos += Vec3::Y * floor_height;
                }
                spawn_floor(commands, info.pos);
                if candidate.lift_floor {
                    info.pos -= Vec3::Y * floor_height;
                }
            }

            if candidate.lift_entity {
                info.pos += Vec3::Y * entity_height;
            }

            (candidate.spawn)(commands, &info);
        }
    }

    std::mem::drop(color_spawners);

    // Spawn ziplines
    spawn_ziplines(
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
                    rooms: junction_to_levels[&hallway_pattern.pattern].clone(),
                },
                SpawnPoint {}, // Hallways are also spawn points
            ));
        }
    }

    spawn_chains(&level_tag, commands, common, &chains.lock().unwrap());
}

fn spawn_ziplines(
    level_tag: &LevelTag,
    commands: &mut Commands,
    common: &Common,
    zipline_positions: &HashMap<IVec2, Vec3>,
) {
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
    for &p in zipline_positions.keys() {
        if visited.contains(&p) {
            continue;
        }

        if neighbors8(p)
            .into_iter()
            .filter(|neighbor| zipline_positions.contains_key(neighbor))
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
                if !visited.contains(&neighbor) && zipline_positions.contains_key(&neighbor) {
                    visited.insert(neighbor);
                    region.push(neighbor);
                }
            }
            i += 1;
        }

        spawn_zipline(level_tag, commands, common, &region, zipline_positions);
    }
}

/// Spawn a single zipline, in order.
fn spawn_zipline(
    level_tag: &LevelTag,
    commands: &mut Commands,
    common: &Common,
    zipline_positions: &[IVec2],
    world_positions: &HashMap<IVec2, Vec3>,
) {
    let mut nodes: Vec<Vec3> = Vec::new();
    for i in 0..zipline_positions.len() - 1 {
        let a = zipline_positions[i];
        let b = zipline_positions[i + 1];

        let end_a = world_positions[&a];
        let end_b = world_positions[&b];

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

fn spawn_chains(
    level_tag: &LevelTag,
    commands: &mut Commands,
    common: &Common,
    chain_positions: &HashMap<IVec2, Vec3>,
) {
    let mut chain_entities: HashMap<IVec2, Entity> = default();
    for (chain_ball, chain_pos) in chain_positions.iter() {
        let collision_layer = if (chain_ball.x + chain_ball.y) % 2 == 0 {
            let mut interact = LayerMask::ALL;
            interact.remove(4);
            CollisionLayers::new(2, interact)
        } else {
            let mut interact = LayerMask::ALL;
            interact.remove(2);
            CollisionLayers::new(4, interact)
        };
        let mut chain_spawned = commands.spawn((
            level_tag.clone(),
            Mesh3d(common.mesh_small_sphere.clone()),
            MeshMaterial3d(common.material_dark_gray.clone()),
            Transform::from_translation(*chain_pos).with_scale(Vec3::splat(0.75)),
            RigidBody::Dynamic,
            ColliderDensity(0.1),
            Collider::sphere(0.5),
            collision_layer,
        ));

        if [IVec2::X, IVec2::NEG_X, IVec2::Y, IVec2::NEG_Y]
            .into_iter()
            .filter(|d| chain_positions.contains_key(&(*chain_ball + *d)))
            .count()
            == 1
        {
            // The ends of the chain are draggable.
            chain_spawned.insert((
                Draggable::default(),
                Collider::cuboid(1., 1., 1.),
                Mesh3d(common.mesh_cube.clone()),
                Transform::from_translation(*chain_pos).with_scale(Vec3::splat(0.6)),
                MeshMaterial3d(common.material_orange.clone()),
                Plug::default(),
            ));
        }

        let chain_id = chain_spawned.id();

        chain_entities.insert(*chain_ball, chain_id);
    }

    for chain_ball in chain_entities.keys() {
        for dx in -1..=1 {
            for dz in -1..=1 {
                if (dx, dz) <= (0, 0) {
                    continue;
                }
                if dx != 0 && dz != 0 {
                    continue;
                }
                let other = chain_ball + IVec2::new(dx, dz);
                if !chain_entities.contains_key(&other) {
                    continue;
                }
                // Add a constraint between them.
                let delta = chain_positions[&other] - chain_positions[chain_ball];

                commands.spawn((
                    level_tag.clone(),
                    SphericalJoint::new(chain_entities[chain_ball], chain_entities[&other])
                        .with_local_anchor_1(delta / 2.0)
                        .with_local_anchor_2(-delta / 2.0)
                        .with_linear_velocity_damping(0.5)
                        .with_angular_velocity_damping(0.5)
                        .with_compliance(0.05),
                    ChainLink(chain_entities[chain_ball], chain_entities[&other]),
                    Mesh3d(common.mesh_cube.clone()),
                    MeshMaterial3d(common.material_dark_gray.clone()),
                ));
            }
        }
    }
}
