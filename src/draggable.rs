use avian3d::prelude::*;
use bevy::prelude::*;

use crate::{
    interactible::{Activated, Interactible},
    player::Player,
};

const PICK_RADIUS: f32 = 1.9;
const DROP_RADIUS: f32 = 2.5;

#[derive(Default, Component)]
#[require(Interactible = Interactible::radius(PICK_RADIUS))]
#[require(ExternalForce)]
pub struct Draggable {
    pub is_dragging: bool,
}

pub struct DraggablePlugin;

impl Plugin for DraggablePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                pick_draggable_system,
                update_draggable_system,
                run_draggable_system,
            )
                .chain(),
        );
    }
}
pub fn pick_draggable_system(mut draggable: Query<(&mut Draggable, &mut Activated)>) {
    for (mut draggable, mut activated) in draggable.iter_mut() {
        if activated.take_activated() {
            draggable.is_dragging = !draggable.is_dragging;
        }
    }
}

pub fn update_draggable_system(
    mut draggable: Query<(&Draggable, &mut Interactible), Changed<Draggable>>,
) {
    for (draggable, mut interactive) in draggable.iter_mut() {
        if !draggable.is_dragging {
            interactive.priority = 0;
            interactive.radius = PICK_RADIUS;
        } else {
            interactive.priority = 5;
            interactive.radius = DROP_RADIUS;
        }
    }
}

pub fn run_draggable_system(
    player: Query<(&Player, &Transform)>,
    mut draggable: Query<(
        &mut Draggable,
        &Transform,
        &mut ExternalForce,
        &LinearVelocity,
    )>,
) {
    for (mut draggable, draggable_transform, mut force, velocity) in draggable.iter_mut() {
        force.set_force(Vec3::ZERO);
        if !draggable.is_dragging {
            continue;
        }

        let Ok((_, player_transform)) = player.single() else {
            continue;
        };

        let target_position = player_transform.translation;
        // The ideal distance between the player and the thing they're dragging.
        let target_distance = 1.2;

        let target_position: Vec3 = target_position
            + (draggable_transform.translation - target_position).normalize() * target_distance;

        let mut delta = target_position - draggable_transform.translation;

        if delta.length() >= DROP_RADIUS - target_distance {
            // If too far away, snap the connection.
            draggable.is_dragging = false;
            continue;
        }

        // The strength of the pulling/pushing force, at the max distance.
        let pull_strength = 12.;
        // The strength of XZ velocity damping.
        let damp_strength = 0.5;
        // The distance at which pulling strength reaches 100%.
        let pull_radius = 1.2;

        delta /= pull_radius;
        if delta.length() > 1. {
            delta = delta.normalize();
        }

        force
            .set_force(**velocity * Vec3::new(1., 0., 1.) * -damp_strength + pull_strength * delta);
    }
}
