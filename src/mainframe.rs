use bevy::prelude::*;

use crate::{
    common::Common,
    interactible::{Activated, Interactible},
};

fn mainframe_point_light() -> PointLight {
    PointLight {
        range: 3.,
        radius: 0.3,
        intensity: 0.,
        color: Color::linear_rgb(0.2, 0.5, 1.0),
        ..default()
    }
}

#[derive(Component)]
#[require(Interactible = Interactible::radius(1.9).with_priority(2), PointLight = mainframe_point_light())]
pub struct Mainframe {
    /// Whether the player has activate the mainframe.
    pub active: bool,
    pub has_charge: bool,
}

pub struct MainframePlugin;

impl Plugin for MainframePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                set_mainframe_icon_system.after(crate::electricity::compute_charge_system),
                activate_computer_system,
                recolor_computer,
            )
                .chain(),
        );
    }
}

pub fn set_mainframe_icon_system(
    mut mainframe: Query<(&mut Interactible, &Mainframe)>,
    common: Res<Common>,
) {
    for (mut interactible, mainframe) in mainframe.iter_mut() {
        let expected_icon = if mainframe.has_charge {
            &common.material_icon_e
        } else {
            &common.material_icon_low_power
        };
        if interactible.icon.as_ref() != Some(expected_icon) {
            interactible.icon = Some(expected_icon.clone());
        }
    }
}

pub fn activate_computer_system(
    mut commands: Commands,
    mut mainframe: Query<(Entity, &mut Mainframe, &mut Activated)>,
) {
    for (entity, mut mainframe, mut activated) in mainframe.iter_mut() {
        if activated.take_activated() && mainframe.has_charge {
            mainframe.active = !mainframe.active;
            commands.entity(entity).remove::<Interactible>();
        }
    }
}

pub fn recolor_computer(
    mut commands: Commands,
    mut mainframe: Query<(Entity, &Mainframe, &mut PointLight), Changed<Mainframe>>,
    common: Res<Common>,
) {
    for (entity, mainframe, mut light) in mainframe.iter_mut() {
        if mainframe.active {
            commands
                .entity(entity)
                .insert(MeshMaterial3d(common.material_active.clone()));
            light.intensity = light_consts::lux::RAW_SUNLIGHT;
        } else {
            commands
                .entity(entity)
                .insert(MeshMaterial3d(common.material_yellow.clone()));

            light.intensity = 0.0;
        }
    }
}

use crate::door::{Door, DoorTrigger};

pub fn unlock_doors_when_all_mainframes_active(
    mut commands: Commands,
    mainframes: Query<&Mainframe>,
    doors: Query<(Entity, &Transform), With<Door>>,
) {
    if mainframes.iter().all(|mf| mf.active) {
        for (door_entity, transform) in doors.iter() {
            // Despawn the visible door
            commands.entity(door_entity).despawn();

            // Spawn a door trigger in its place
            commands.spawn((
                Transform::from_translation(transform.translation),
                GlobalTransform::default(),
                DoorTrigger,
            ));
        }
    }
}
