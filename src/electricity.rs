use avian3d::prelude::*;
use bevy::{
    platform::collections::{HashMap, HashSet},
    prelude::*,
};

use crate::{
    common::Common,
    draggable::Draggable,
    interactible::{Activated, Interactible},
    mainframe::Mainframe,
};

pub struct ElectricityPlugin;

impl Plugin for ElectricityPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PowerGrid {
            active: HashSet::new(),
        });
        app.add_systems(
            FixedUpdate,
            (
                outlet_interactive_system.after(crate::draggable::run_draggable_system),
                plug_physics_system.after(crate::draggable::run_draggable_system),
                compute_charge_system,
                visible_wire_system,
            )
                .chain(),
        );
    }
}

/// This component currently has electricity.
#[derive(Component)]
pub struct Charged;

#[derive(Component)]
#[require(ExternalForce)]
pub struct Plug {
    pub outlet: Option<Entity>,
    pub other_end: Entity,
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
                .insert(Interactible::radius(1.2).with_priority(6));
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

#[derive(Component)]
pub struct PowerSource;

#[derive(Component)]
pub struct Wire;

#[derive(Resource)]
/// It's a pun, get it?
pub struct PowerGrid {
    pub active: HashSet<IVec2>,
}

pub fn global_to_grid(p: Vec3) -> IVec2 {
    p.round().xz().as_ivec2()
}

pub fn compute_charge_system(
    mut power_grid: ResMut<PowerGrid>,
    sources: Query<(&GlobalTransform, &PowerSource)>,
    outlets: Query<&GlobalTransform, With<Outlet>>,
    wires: Query<&GlobalTransform, With<Wire>>,
    mut mainframes: Query<(&GlobalTransform, &mut Mainframe)>,
    plugs: Query<&Plug>,
) {
    power_grid.active.clear();

    for (transform, _source) in sources.iter() {
        power_grid
            .active
            .insert(global_to_grid(transform.translation()));
    }

    let mut mainframes_to_charge: HashMap<IVec2, Mut<Mainframe>> = HashMap::new();

    for (transform, mut mainframe) in mainframes.iter_mut() {
        if !mainframe.active {
            mainframe.has_charge = false;
            mainframes_to_charge.insert(global_to_grid(transform.translation()), mainframe);
            continue;
        }
        power_grid
            .active
            .insert(global_to_grid(transform.translation()));
    }

    let mut wire_grid: HashSet<IVec2> = HashSet::new();
    for wire_transform in wires.iter() {
        wire_grid.insert(global_to_grid(wire_transform.translation()));
    }
    for outlet_transform in outlets.iter() {
        wire_grid.insert(global_to_grid(outlet_transform.translation()));
    }

    let mut plug_connections: HashMap<IVec2, IVec2> = HashMap::new();
    for plug in plugs.iter() {
        if let Ok(plug_other) = plugs.get(plug.other_end) {
            // If both ends are plugged in, connect them in the grid.
            if let Some(outlet) = plug.outlet {
                if let Some(outlet_other) = plug_other.outlet {
                    if let Ok(outlet_transform) = outlets.get(outlet) {
                        if let Ok(other_outlet_transform) = outlets.get(outlet_other) {
                            plug_connections.insert(
                                global_to_grid(outlet_transform.translation()),
                                global_to_grid(other_outlet_transform.translation()),
                            );
                        }
                    }
                }
            }
        }
    }

    let mut stack: Vec<IVec2> = power_grid.active.iter().copied().collect();
    while let Some(p) = stack.pop() {
        // Expand charge to neighboring cells.
        for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let neighbor = p + IVec2::new(dx, dz);
            if let Some(mainframe) = mainframes_to_charge.get_mut(&neighbor) {
                mainframe.has_charge = true;
            }
            if wire_grid.contains(&neighbor) && !power_grid.active.contains(&neighbor) {
                power_grid.active.insert(neighbor);
                stack.push(neighbor);
            }
        }
        // Expand charge to cable-connected cells.
        if let Some(other) = plug_connections.get(&p) {
            if !power_grid.active.contains(other) {
                power_grid.active.insert(*other);
                stack.push(*other);
            }
        }
    }
}

fn visible_wire_system(
    common: Res<Common>,
    mut wire: Query<(&GlobalTransform, &mut MeshMaterial3d<StandardMaterial>), With<Wire>>,
    grid: Res<PowerGrid>,
) {
    for (wire_transform, mut material) in wire.iter_mut() {
        let wire_grid = global_to_grid(wire_transform.translation());
        let is_powered = grid.active.contains(&wire_grid);
        let expected_material = if is_powered {
            &common.material_electricity
        } else {
            &common.material_dark_blue
        };
        if &material.0 != expected_material {
            material.0 = expected_material.clone();
        }
    }
}
