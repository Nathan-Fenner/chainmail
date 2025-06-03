use bevy::prelude::*;

use crate::player::Player;

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
    robots: Query<(&EvilRobot, &Transform), Without<Player>>,
) {
    let Ok((player, mut player_transform)) = player.single_mut() else {
        return;
    };

    for (robot, pos) in robots.iter() {
        if (pos.translation.distance(player_transform.translation) < 1.2) {
            player_transform.translation = vec3(1.0, 2.0, 1.0)
        }
    }
}
