use bevy::prelude::*;

use crate::{common::Common, player::Player, spawn_point::CurrentSpawnPoint};

#[derive(Component)]
pub struct EvilRobot {
    pub has_charge: bool,
    pub active: bool,
}

// query for obot mesh3d mat
// check has charge
// read from, res comm
// replace mesh3 mat

pub struct EvilRobotPlugin;

impl Plugin for EvilRobotPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, captured);
    }
}

pub fn update_evil_robot_materials(
    mut robots: Query<(&EvilRobot, &mut MeshMaterial3d<StandardMaterial>)>,
    common: Res<Common>,
) {
    for (robot, mut material) in robots.iter_mut() {
        if robot.has_charge {
            *material = MeshMaterial3d(common.material_zappy_boy.clone());
        } else {
            *material = MeshMaterial3d(common.material_beepboop.clone());
        }
    }
}

fn captured(
    mut player: Query<(&Player, &mut Transform)>,
    robots: Query<(&EvilRobot, &Transform), (With<EvilRobot>, Without<Player>)>,
    current_spawn: Res<CurrentSpawnPoint>,
) {
    let Ok((_player, mut player_transform)) = player.single_mut() else {
        return;
    };

    for (robot_entity, robot_transform) in robots.iter() {
        if robot_entity.has_charge
            && robot_transform
                .translation
                .distance(player_transform.translation)
                < 1.2
        {
            player_transform.translation = current_spawn.location;
        }
    }
}
