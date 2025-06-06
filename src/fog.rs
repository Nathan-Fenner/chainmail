use bevy::{platform::collections::HashSet, prelude::*};
use rand::Rng;

use crate::{common::Common, player::Player};

pub struct FogPlugin;

impl Plugin for FogPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_fog_system.after(crate::common::setup_common))
            .add_systems(Update, clear_fog_system);
    }
}

#[derive(Component)]
struct Fog {
    speed: f32,
}

const GRID_SPACING: f32 = 2.3;
const FOG_GRID_SIZE: i32 = 50;

fn spawn_fog_system(mut commands: Commands, common: Res<Common>) {
    let mut rand = rand::rng();
    for x in 0..FOG_GRID_SIZE {
        for z in 0..FOG_GRID_SIZE {
            commands.spawn((
                Mesh3d(common.mesh_cube.clone()),
                MeshMaterial3d(common.material_fog.clone()),
                Fog {
                    speed: rand.random_range(0.03..0.1),
                },
                Transform::from_translation(Vec3::new(x as f32, 0.0, z as f32) * GRID_SPACING)
                    .looking_to(
                        Vec3::new(
                            rand.random_range(-1.0..1.0),
                            rand.random_range(-0.1..0.1),
                            rand.random_range(-1.0..1.0),
                        ),
                        Vec3::Y,
                    )
                    .with_scale(Vec3::splat(1.5)),
            ));
        }
    }
}

fn clear_fog_system(
    mut fog: Query<(&mut Transform, &Fog)>,
    player: Query<&GlobalTransform, With<Player>>,
    clear: Query<&GlobalTransform, Without<Fog>>,
) {
    let mut grid_to_clear: HashSet<IVec2> = HashSet::new();
    for p in clear.iter() {
        grid_to_clear.insert(p.translation().xz().round().as_ivec2());
    }

    let Ok(player) = player.single() else {
        return;
    };

    let player = player.translation();

    const FOG_SIZE: f32 = GRID_SPACING * FOG_GRID_SIZE as f32;

    let max_scale = 3.0;

    for (mut fog, fog_speed) in fog.iter_mut() {
        if fog.translation.x < player.x - FOG_SIZE / 2. {
            fog.translation.x += FOG_SIZE;
            fog.scale = Vec3::splat(max_scale);
        }
        if fog.translation.z < player.z - FOG_SIZE / 2. {
            fog.translation.z += FOG_SIZE;
            fog.scale = Vec3::splat(max_scale);
        }
        if fog.translation.x > player.x + FOG_SIZE / 2. {
            fog.translation.x -= FOG_SIZE;
            fog.scale = Vec3::splat(max_scale);
        }
        if fog.translation.z > player.z + FOG_SIZE / 2. {
            fog.translation.z -= FOG_SIZE;
            fog.scale = Vec3::splat(max_scale);
        }

        let p = fog.translation.xz().round().as_ivec2();
        let target_scale = if grid_to_clear.contains(&p) {
            0.0
        } else {
            max_scale
        };

        // let fog_distance = player.xz().distance(fog.translation.xz());

        fog.scale = fog.scale.lerp(Vec3::splat(target_scale), fog_speed.speed);
        // fog.scale.y = 3.2;
    }
}
