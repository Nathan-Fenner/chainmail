use bevy::prelude::*;

use crate::player::Player;

/// A component for things which can be interacted with.
#[derive(Component, Debug)]
#[require(Activated = Activated{activated: false})]
pub struct Interactible {
    pub radius: f32,
}

/// Tracks when the user interacts with an item.
#[derive(Component, Default)]
pub struct Activated {
    pub activated: bool,
}

impl Activated {
    /// Returns `true` if just activated.
    /// Sets the flag to false.
    pub fn take_activated(&mut self) -> bool {
        let v = self.activated;
        self.activated = false;
        v
    }
}

pub struct InteractiblePlugin;

impl Plugin for InteractiblePlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.insert_resource(NearestInteractible {
            entity: None,
            size: 1.,
            velocity: 0.,
        })
        .add_systems(
            Update,
            (
                set_nearest_interactible_system,
                visualize_interactible_system,
                mark_activated_system,
            )
                .chain(),
        );
    }
}

#[derive(Resource)]
pub struct NearestInteractible {
    entity: Option<Entity>,
    size: f32,
    velocity: f32,
}

/// Sets `NearestInteractible` to the entity closest to the player.
fn set_nearest_interactible_system(
    player: Query<&GlobalTransform, With<Player>>,
    interactibles: Query<(Entity, &GlobalTransform, &Interactible)>,
    mut nearest_interactible: ResMut<NearestInteractible>,
) {
    let found_nearest: Option<Entity> = (move || {
        let Ok(player_transform) = player.single() else {
            return None;
        };

        #[derive(Copy, Clone)]
        struct Candidate {
            distance: f32,
            entity: Entity,
        }

        let mut closest: Option<Candidate> = None;
        for (entity, transform, interactible) in interactibles.iter() {
            let distance = player_transform
                .translation()
                .distance(transform.translation())
                - interactible.radius;

            if distance > 0.0 {
                continue;
            }

            if closest.map(|c| c.distance > distance).unwrap_or(true) {
                closest = Some(Candidate { distance, entity });
            }
        }

        closest.map(|c| c.entity)
    })();

    if nearest_interactible.entity != found_nearest {
        nearest_interactible.entity = found_nearest;
        nearest_interactible.size = 0.5;
        nearest_interactible.velocity = 0.0;
    }
}

fn visualize_interactible_system(
    time: Res<Time>,
    mut gizmos: Gizmos,
    mut nearest_state: ResMut<NearestInteractible>,
    global_transform: Query<&GlobalTransform>,
) {
    let delta = time.delta_secs();
    // TODO: animate this
    let Some(nearest) = nearest_state.entity else {
        return;
    };

    let Ok(transform) = global_transform.get(nearest) else {
        return;
    };

    nearest_state.size += nearest_state.velocity * delta;
    nearest_state.velocity *= (0.05f32).powf(delta);
    nearest_state.velocity += (1.0 - nearest_state.size) * 100. * delta;

    gizmos.sphere(
        transform.translation() + Vec3::Y,
        0.15 * nearest_state.size,
        Color::linear_rgb(1., 1., 1.),
    );
}

pub fn mark_activated_system(
    mut nearest_state: ResMut<NearestInteractible>,
    mut activated: Query<&mut Activated>,
    key: Res<ButtonInput<KeyCode>>,
) {
    if !key.just_pressed(KeyCode::KeyE) {
        return;
    }
    let Some(nearest) = nearest_state.entity else {
        return;
    };

    let Ok(mut nearest) = activated.get_mut(nearest) else {
        return;
    };
    nearest.activated = true;
    nearest_state.size = 1.;
    nearest_state.velocity = 10.;
}
