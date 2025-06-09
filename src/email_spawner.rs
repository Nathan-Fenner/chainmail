use bevy::prelude::*;

use crate::{common::Common, fog::DoesNotClearFog, level::LevelTag};

pub struct EmailSpawnerPlugin;

impl Plugin for EmailSpawnerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (spawn_emails, move_particles));
    }
}

#[derive(Component)]
pub struct EmailSpawner {
    pub rate: f32,
    pub debounce: f32,
    pub dir: f32,
    pub scale: f32,
}

#[derive(Component)]
pub struct Particle {
    time_left: f32,
    velocity: Vec3,
}

pub fn spawn_emails(
    mut commands: Commands,
    time: Res<Time>,
    mut spawners: Query<(&GlobalTransform, &mut EmailSpawner, &LevelTag)>,
    common: Res<Common>,
) {
    let delta = time.delta_secs();
    for (transform, mut spawner, level_tag) in spawners.iter_mut() {
        spawner.debounce -= delta;
        if spawner.debounce <= 0.0 {
            spawner.debounce = spawner.rate;
            // Spawn!

            let velocity =
                Vec3::new(spawner.dir.cos(), 9.0, spawner.dir.sin()) * 1. * spawner.scale;

            spawner.dir += 2.0;

            commands.spawn((
                Particle {
                    time_left: 2.0,
                    velocity,
                },
                MeshMaterial3d(common.material_email.clone()),
                Mesh3d(common.mesh_plane.clone()),
                Transform::from_translation(transform.translation())
                    .with_scale(Vec3::splat(spawner.scale)),
                level_tag.clone(),
                DoesNotClearFog,
            ));
        }
    }
}

pub fn move_particles(
    time: Res<Time>,
    mut commands: Commands,
    mut particles: Query<(Entity, &mut Transform, &mut Particle)>,
) {
    let delta = time.delta_secs();
    for (entity, mut transform, mut particle) in particles.iter_mut() {
        particle.time_left -= delta;
        if particle.time_left <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }
        transform.translation += particle.velocity * delta;
        particle.velocity *= (0.5f32).powf(delta);
        particle.velocity -= Vec3::Y * 4. * delta;

        transform.rotate_local_x(delta * 5.0);
        transform.rotate_local_y(delta * 2.0);
    }
}
