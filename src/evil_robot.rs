use bevy::prelude::*;

use crate::{
    common::Common, electricity::PowerGrid, player::Player, spawn_point::CurrentSpawnPoint,
};

#[derive(Component)]
pub struct EvilRobot {
    pub has_charge: bool,
}

// query for obot mesh3d mat
// check has charge
// read from, res comm
// replace mesh3 mat

pub struct EvilRobotPlugin;

impl Plugin for EvilRobotPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                set_robot_charge_system.after(crate::electricity::compute_charge_system),
                capture_player_system,
            )
                .chain(),
        )
        .add_systems(Update, (update_evil_robot_materials, spinning_system));
    }
}

fn set_robot_charge_system(
    power_grid: Res<PowerGrid>,
    mut robot: Query<(&GlobalTransform, &mut EvilRobot)>,
) {
    for (robot_transform, mut robot) in robot.iter_mut() {
        let grid_position = robot_transform.translation().xz().round().as_ivec2();
        robot.has_charge = power_grid.active.contains(&grid_position);
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

fn capture_player_system(
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
                < 2.4
        {
            player_transform.translation = current_spawn.location;
        }
    }
}
#[derive(Component)]
pub struct Spinning(pub Vec3);

pub fn spinning_system(
    time: Res<Time>,
    mut spinning: Query<(&GlobalTransform, &mut Transform, &Spinning)>,
    power: Res<PowerGrid>,
) {
    let delta = time.delta_secs();
    for (global_transform, mut transform, spinning) in spinning.iter_mut() {
        let grid = global_transform.translation().xz().round().as_ivec2();
        let has_power = power.active.contains(&grid);

        let target_scale = if has_power {
            Vec3::splat(4.)
        } else {
            Vec3::splat(0.)
        };

        transform.rotate_axis(
            Dir3::try_from(spinning.0).unwrap(),
            spinning.0.length() * delta,
        );
        transform.rotation = transform.rotation.normalize();
        transform.scale = transform
            .scale
            .lerp(target_scale, (10.0 * delta).clamp(0.0, 0.5));
    }
}
