use bevy::prelude::*;

use crate::{player::Player, spawn_point::CurrentSpawnPoint};

#[derive(Component)]
pub struct EvilRobot {}

pub struct EvilRobotPlugin;

impl Plugin for EvilRobotPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, captured);
    }
}

fn captured(
    mut player: Query<(&Player, &mut Transform)>,
    robots: Query<&Transform, (With<EvilRobot>, Without<Player>)>,
    current_spawn: Res<CurrentSpawnPoint>,
) {
    let Ok((_player, mut player_transform)) = player.single_mut() else {
        return;
    };

    for robot_transform in robots.iter() {
        if robot_transform
            .translation
            .distance(player_transform.translation)
            < 1.2
        {
            player_transform.translation = current_spawn.location;
        }
    }
}
