use avian3d::prelude::*;
use bevy::prelude::*;

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, (control_player,))
            .add_systems(Update, control_camera_system);
    }
}

#[derive(Component)]
#[require(ExternalForce, GravityScale, RecentVelocity = RecentVelocity{direction: Vec3::X})]
pub struct Player {}

#[derive(Component)]
pub struct RecentVelocity {
    // This will be a non-zero value, indicating the player's recent velocity.
    pub direction: Vec3,
}

#[derive(Component)]
pub struct PlayerCamera;

pub fn control_player(
    mut player: Query<(
        &mut Player,
        &mut ExternalForce,
        &LinearVelocity,
        &mut RecentVelocity,
    )>,
    camera: Query<(&Transform, &PlayerCamera), Without<Player>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let Ok(camera) = camera.single() else {
        return;
    };

    let mut keydir = Vec2::new(0., 0.);
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
        keydir.x = -1.;
    }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
        keydir.x = 1.;
    }
    if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
        keydir.y = 1.;
    }
    if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) {
        keydir.y = -1.;
    }
    if keydir.length() > 1. {
        keydir = keydir.normalize();
    }

    let forward = (camera.0.forward().as_vec3() * Vec3::new(1., 0., 1.)).normalize();
    let right = forward.cross(Vec3::Y);

    let target_speed = 6.;

    let desired_velocity = (keydir.x * right + keydir.y * forward) * target_speed;

    let damp_strength = 0.4;
    let accel_strength = 6.0;

    for (_player, mut player_force, player_velocity, mut recent_velocity) in player.iter_mut() {
        let difference: Vec3 = desired_velocity - **player_velocity;

        player_force.set_force(
            -**player_velocity * damp_strength
                + difference * Vec3::new(1., 0., 1.) * accel_strength,
        );

        if player_velocity.length() > 0.1 {
            recent_velocity.direction = recent_velocity.direction.lerp(**player_velocity, 0.2);
            if recent_velocity.direction.length() < 0.1 {
                recent_velocity.direction = recent_velocity.direction.normalize_or_zero() * 0.1;
            }
        }
    }
    // ...
}

fn control_camera_system(
    player: Query<&Transform, With<Player>>,
    mut camera: Query<&mut Transform, (With<PlayerCamera>, Without<Player>)>,
) {
    let Ok(player) = player.single() else {
        return;
    };

    let Ok(mut camera) = camera.single_mut() else {
        return;
    };

    let player_ground = player.translation * Vec3::new(1.0, 0.0, 1.0) + Vec3::Y;
    camera.translation = player_ground + Vec3::new(0., 22., 14.);
    camera.look_at(player_ground, Vec3::Y);
}
