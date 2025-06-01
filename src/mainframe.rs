use bevy::prelude::*;

use crate::{
    common::Common,
    interactible::{Activated, Interactible},
};

#[derive(Component)]
#[require(Interactible = Interactible::radius(1.9))]
pub struct Mainframe {
    /// Whether the player has activate the mainframe.
    pub active: bool,
}

pub struct MainframePlugin;

impl Plugin for MainframePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (activate_computer_system, recolor_computer).chain());
    }
}

pub fn activate_computer_system(mut mainframe: Query<(&mut Mainframe, &mut Activated)>) {
    for (mut mainframe, mut activated) in mainframe.iter_mut() {
        if activated.take_activated() {
            mainframe.active = !mainframe.active;
        }
    }
}

pub fn recolor_computer(
    mut commands: Commands,
    mainframe: Query<(Entity, &Mainframe), Changed<Mainframe>>,
    common: Res<Common>,
) {
    for (entity, mainframe) in mainframe.iter() {
        println!("visit compute {:?}", entity);
        if mainframe.active {
            commands
                .entity(entity)
                .insert(MeshMaterial3d(common.material_active.clone()));
        } else {
            commands
                .entity(entity)
                .insert(MeshMaterial3d(common.material_yellow.clone()));
        }
    }
}
