use avian3d::prelude::*;
use bevy::prelude::*;

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, control_player);
    }
}

#[derive(Component)]
#[require(ExternalForce, GravityScale)]
pub struct Player {}

#[derive(Component)]
pub struct PlayerCamera;

fn control_player(
    mut player: Query<(&mut Player, &mut ExternalForce, &LinearVelocity)>,
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

    let damp_strength = 0.5;
    let accel_strength = 10.0;

    for (_player, mut player_force, player_velocity) in player.iter_mut() {
        let difference: Vec3 = desired_velocity - **player_velocity;

        player_force.set_force(
            -**player_velocity * damp_strength
                + difference * Vec3::new(1., 0., 1.) * accel_strength,
        );
    }
    // ...
}
