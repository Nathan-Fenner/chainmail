use bevy::prelude::*;

use crate::player::Player;
use crate::spawn_point::CurrentSpawnPoint;

#[derive(Component)]
pub struct Well;

/// Threshold below which the player is considered to have "fallen"
const FALL_Y_THRESHOLD: f32 = -1.0;

pub struct WellPlugin;

impl Plugin for WellPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, check_well_fall);
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
