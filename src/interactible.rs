use bevy::prelude::*;

use crate::{
    common::{Common, setup_common},
    player::{Player, PlayerCamera},
    ruby::CanZip,
};

/// A component for things which can be interacted with.
#[derive(Component, Debug)]
#[require(Activated = Activated{activated: false})]
pub struct Interactible {
    pub radius: f32,
    pub priority: i32,
    pub dot_offset: Vec3,

    pub icon: Option<Handle<StandardMaterial>>,
    pub needs_ruby: bool,
}

impl Interactible {
    /// Create a new `Interactible` with the specified radius and 0 priority.
    pub fn radius(radius: f32) -> Self {
        Self {
            radius,
            priority: 0,
            dot_offset: Vec3::ZERO,
            icon: None,
            needs_ruby: false,
        }
    }
    /// Update the priority of the input.
    pub fn with_priority(self, priority: i32) -> Self {
        Self { priority, ..self }
    }
    /// Sets the icon for this interactible.
    pub fn with_icon(self, icon: Handle<StandardMaterial>) -> Self {
        Self {
            icon: Some(icon),
            ..self
        }
    }
    pub fn with_dot_offset(self, offset: Vec3) -> Self {
        Self {
            dot_offset: offset,
            ..self
        }
    }
    pub fn needs_ruby(self) -> Self {
        Self {
            needs_ruby: true,
            ..self
        }
    }
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
            icon: None,
        })
        .add_systems(Startup, spawn_interactible_dot_system.after(setup_common))
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
struct InteractibleDot {
    dot_entity: Entity,
}

fn spawn_interactible_dot_system(mut commands: Commands, common: Res<Common>) {
    let dot_entity = commands
        .spawn((
            Mesh3d(common.mesh_plane.clone()),
            MeshMaterial3d(common.material_icon_e.clone()),
        ))
        .id();

    commands.insert_resource(InteractibleDot { dot_entity });
}

#[derive(Resource)]
pub struct NearestInteractible {
    entity: Option<Entity>,
    size: f32,
    velocity: f32,
    icon: Option<Handle<StandardMaterial>>,
}

/// Sets `NearestInteractible` to the entity closest to the player.
fn set_nearest_interactible_system(
    player: Query<&GlobalTransform, With<Player>>,
    interactibles: Query<(Entity, &GlobalTransform, &Interactible)>,
    mut nearest_interactible: ResMut<NearestInteractible>,
    has_ruby: Res<CanZip>,
) {
    let found_nearest: Option<Entity> = (move || {
        let Ok(player_transform) = player.single() else {
            return None;
        };

        #[derive(Copy, Clone)]
        struct Candidate {
            distance: f32,
            priority: i32,
            entity: Entity,
        }

        let mut closest: Option<Candidate> = None;
        for (entity, transform, interactible) in interactibles.iter() {
            if interactible.needs_ruby && !has_ruby.can_zip {
                // Skip this before collect ruby
                continue;
            }
            let distance = player_transform
                .translation()
                .distance(transform.translation())
                - interactible.radius;

            if distance > 0.0 {
                continue;
            }

            if closest
                .map(|c| (-c.priority, c.distance) > (-interactible.priority, distance))
                .unwrap_or(true)
            {
                closest = Some(Candidate {
                    distance,
                    priority: interactible.priority,
                    entity,
                });
            }
        }

        closest.map(|c| c.entity)
    })();

    if nearest_interactible.entity != found_nearest {
        nearest_interactible.entity = found_nearest;
        nearest_interactible.size = 0.5;
        nearest_interactible.velocity = 0.0;
        nearest_interactible.icon = found_nearest
            .and_then(|found_nearest| interactibles.get(found_nearest).unwrap().2.icon.clone());
    }
}

fn visualize_interactible_system(
    time: Res<Time>,
    mut nearest_state: ResMut<NearestInteractible>,
    global_transform: Query<&GlobalTransform>,
    mut dot_transform: Query<&mut Transform>,
    mut dot_material: Query<&mut MeshMaterial3d<StandardMaterial>>,
    interactible: Query<&Interactible>,
    the_dot: Res<InteractibleDot>,
    camera: Query<&GlobalTransform, With<PlayerCamera>>,
    common: Res<Common>,
) {
    let Ok(mut dot_transform) = dot_transform.get_mut(the_dot.dot_entity) else {
        return;
    };
    let Ok(mut dot_material) = dot_material.get_mut(the_dot.dot_entity) else {
        return;
    };

    let Ok(camera) = camera.single() else {
        return;
    };

    dot_transform.scale = Vec3::splat(0.0);

    let delta = time.delta_secs();
    // TODO: animate this
    let Some(nearest) = nearest_state.entity else {
        return;
    };

    let Ok(transform) = global_transform.get(nearest) else {
        return;
    };
    let Ok(nearest_config) = interactible.get(nearest) else {
        return;
    };

    nearest_state.size += nearest_state.velocity * delta;
    nearest_state.velocity *= (0.05f32).powf(delta);
    nearest_state.velocity += (1.0 - nearest_state.size) * 100. * delta;

    let expected_material = nearest_state
        .icon
        .as_ref()
        .unwrap_or(&common.material_icon_e);

    if &dot_material.0 != expected_material {
        dot_material.0 = expected_material.clone();
    }

    dot_transform.translation = transform.translation() + Vec3::Y + nearest_config.dot_offset;
    dot_transform.scale = Vec3::splat(0.6 * nearest_state.size);
    let to_camera = (camera.translation() - dot_transform.translation).normalize();
    dot_transform.translation += to_camera * 1.0;
    dot_transform.look_at(camera.translation(), -Vec3::Y);
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
