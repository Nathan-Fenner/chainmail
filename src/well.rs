use bevy::prelude::*;

use crate::player::Player;
use crate::spawn_point::CurrentSpawnPoint;

#[derive(Component)]
pub struct Well;

/// Threshold below which the player is considered to have "fallen"
const FALL_Y_THRESHOLD: f32 = -1.0;

pub struct WellPlugin;

#[derive(Component)]
pub struct DespawnFalling;

impl Plugin for WellPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, (check_well_fall, despawn_falling_system));
    }
}

pub fn check_well_fall(
    mut queries: ParamSet<(
        Query<(&Player, &mut Transform)>,
        Query<&Transform, With<Well>>,
    )>,
    current_spawn: Res<CurrentSpawnPoint>,
) {
    {
        let mut player_query = queries.p0();
        let Ok((_player, mut player_transform)) = player_query.single_mut() else {
            return;
        };

        // Y-position fall check
        if player_transform.translation.y < FALL_Y_THRESHOLD {
            player_transform.translation = current_spawn.location;
        }
    }
}

pub fn despawn_falling_system(
    mut commands: Commands,
    falling: Query<(Entity, &GlobalTransform), With<DespawnFalling>>,
) {
    for (entity, transform) in falling.iter() {
        if transform.translation().y < FALL_Y_THRESHOLD {
            commands.entity(entity).despawn();
        }
    }
}
