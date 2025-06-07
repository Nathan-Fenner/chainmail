use avian3d::prelude::*;
use bevy::prelude::*;

use crate::{
    draggable::Draggable,
    interactible::{Activated, Interactible},
};

pub struct ElectricityPlugin;

impl Plugin for ElectricityPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                outlet_interactive_system.after(crate::draggable::run_draggable_system),
                plug_physics_system.after(crate::draggable::run_draggable_system),
            )
                .chain(),
        );
    }
}

/// This component currently has electricity.
#[derive(Component)]
pub struct Charged;

#[derive(Component, Default)]
#[require(ExternalForce)]
pub struct Plug {
    pub outlet: Option<Entity>,
}

#[derive(Component, Default)]
#[require(ExternalForce)]
pub struct Outlet {
    pub plug: Option<Entity>,
}

/// If player picks up a plug, disconnects it from outlet.
/// If player holds a plug, enables all outlets.
pub fn outlet_interactive_system(
    mut commands: Commands,
    outlets_with_interactible: Query<Entity, (With<Outlet>, With<Interactible>)>,
    outlets_without_interactible: Query<Entity, (With<Outlet>, Without<Interactible>)>,
    mut dragging: Query<(Entity, &mut Draggable, &mut Plug)>,
    mut activate_outlet: Query<(Entity, &mut Activated, &mut Outlet)>,
) {
    let mut dragging_plug: Option<(Entity, Mut<Draggable>, Mut<Plug>)> = None;
    for (entity, draggable, mut plug) in dragging.iter_mut() {
        if draggable.is_dragging {
            // Disconnect from outlet
            if let Some(outlet_entity) = plug.outlet {
                plug.outlet = None;
                if let Ok(mut outlet) = activate_outlet.get_mut(outlet_entity) {
                    // Remove the other direction
                    outlet.2.plug = None;
                }
            }
            dragging_plug = Some((entity, draggable, plug));
            break;
        }
    }

    if let Some(mut dragging_plug) = dragging_plug {
        // Enable all outlets without plugs
        for outlet in outlets_without_interactible.iter() {
            if let Ok(outlet) = activate_outlet.get(outlet) {
                if outlet.2.plug.is_some() {
                    // Already has a plug, so it not interactive.
                    continue;
                }
            }
            commands
                .entity(outlet)
                // Higher priority than dropping the dragged item
                .insert(Interactible::radius(2.0).with_priority(6));
        }

        for (outlet_entity, mut activated, mut outlet) in activate_outlet.iter_mut() {
            if activated.take_activated() {
                // Force the player to drop the plug.
                dragging_plug.1.is_dragging = false;
                // Connect the plug and the outlet.
                outlet.plug = Some(dragging_plug.0);
                dragging_plug.2.outlet = Some(outlet_entity);
            }
        }
    } else {
        // Disable all outlets.
        for outlet in outlets_with_interactible.iter() {
            commands.entity(outlet).remove::<Interactible>();
        }
    }
}

fn plug_physics_system(
    transform: Query<&Transform>,
    mut plug: Query<(&Transform, &LinearVelocity, &mut ExternalForce, &Plug)>,
) {
    for (plug_transform, plug_velocity, mut plug_force, plug) in plug.iter_mut() {
        if let Some(outlet_entity) = plug.outlet {
            plug_force.clear();

            let Ok(target) = transform.get(outlet_entity) else {
                continue;
            };
            let target = target.translation + Vec3::Y * 0.5;

            let delta = target - plug_transform.translation;

            plug_force.set_force(delta * 20. - plug_velocity.0 * 0.5);
        }
    }
}
