use bevy::{pbr::NotShadowCaster, platform::collections::HashSet, prelude::*};

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
    progress: f32,
}

#[derive(Component)]
pub struct DoesNotClearFog;

const GRID_SPACING: f32 = 1.0;
const FOG_GRID_SIZE: i32 = 80;

fn spawn_fog_system(mut commands: Commands, common: Res<Common>) {
    for x in 0..FOG_GRID_SIZE {
        for z in 0..FOG_GRID_SIZE {
            commands.spawn((
                Mesh3d(common.mesh_cube.clone()),
                MeshMaterial3d(common.material_fog.clone()),
                Fog { progress: 0.0 },
                NotShadowCaster,
                Transform::from_translation(Vec3::new(x as f32, 0.0, z as f32) * GRID_SPACING)
                    .with_scale(Vec3::new(2.3, 14.0, 2.3)),
            ));
        }
    }
}

fn clear_fog_system(
    time: Res<Time>,
    mut fog: Query<(&mut Transform, &mut Fog)>,
    player: Query<&GlobalTransform, With<Player>>,
    clear: Query<(Entity, &GlobalTransform), (Without<DoesNotClearFog>, Without<Fog>)>,
    parent: Query<&ChildOf>,
    does_not_clear_fog: Query<&DoesNotClearFog>,
) {
    let mut grid_to_clear: HashSet<IVec2> = HashSet::new();
    for (entity, p) in clear.iter() {
        if parent
            .iter_ancestors(entity)
            .any(|ancestor| does_not_clear_fog.contains(ancestor))
        {
            continue;
        }

        let p = p.translation().xz().round().as_ivec2();
        for dx in -1..=1 {
            for dz in -1..=1 {
                grid_to_clear.insert(p + IVec2::new(dx, dz));
            }
        }
    }

    let Ok(player) = player.single() else {
        return;
    };

    let player = player.translation();

    const FOG_SIZE: f32 = GRID_SPACING * FOG_GRID_SIZE as f32;

    let max_scale = 3.0;

    for (mut fog, mut fog_speed) in fog.iter_mut() {
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
        let progress_change = if grid_to_clear.contains(&p) {
            1.0
        } else {
            -1.0
        } * 30.
            * time.delta_secs();

        fog_speed.progress += progress_change;
        fog_speed.progress = fog_speed.progress.clamp(-5.0, 40.0);

        let fog_distance = player.xz().distance(fog.translation.xz());

        let target_scale = if fog_speed.progress > fog_distance {
            0.0
        } else {
            max_scale
        };

        fog.scale = fog
            .scale
            .lerp(Vec3::splat(target_scale) * Vec3::new(1.1, 1.2, 1.1), 0.2);
    }
}
