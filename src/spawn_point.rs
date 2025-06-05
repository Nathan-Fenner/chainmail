use bevy::prelude::*;

use crate::player::Player;

/// Component: marks entities that are spawn points.
#[derive(Component)]
pub struct SpawnPoint;

/// Resource: stores the current active spawn location.
#[derive(Resource, Debug)]
pub struct CurrentSpawnPoint {
    pub location: Vec3,
}

pub struct SpawnPointPlugin;

impl Plugin for SpawnPointPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CurrentSpawnPoint {
            location: Vec3::ZERO, // or a default fallback
        })
        .add_systems(FixedUpdate, update_current_spawn_point);
    }
}

fn update_current_spawn_point(
    player_query: Query<&Transform, With<Player>>,
    spawn_query: Query<&Transform, With<SpawnPoint>>,
    mut current_spawn: ResMut<CurrentSpawnPoint>,
) {
    if let Ok(player_transform) = player_query.single() {
        for spawn_transform in spawn_query.iter() {
            if spawn_transform
                .translation
                .distance(player_transform.translation)
                < 1.2
            {
                current_spawn.location = spawn_transform.translation;
            }
        }
    }
}
