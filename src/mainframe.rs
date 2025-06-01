use bevy::prelude::*;

use crate::{common::Common, player::Player};

#[derive(Component)]
pub struct Mainframe {
    /// Whether the player has activate the mainframe.
    pub active: bool,
}

pub struct MainframePlugin;

impl Plugin for MainframePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (activate_computer, recolor_computer).chain());
    }
}

pub fn activate_computer(
    player: Query<(&GlobalTransform, &Player)>,
    mut mainframe: Query<(&GlobalTransform, &mut Mainframe)>,
    key: Res<ButtonInput<KeyCode>>,
) {
    let Ok((player, _)) = player.single() else {
        return;
    };

    for (mainframe_position, mut mainframe) in mainframe.iter_mut() {
        if key.just_pressed(KeyCode::KeyE)
            && mainframe_position
                .translation()
                .distance(player.translation())
                < 1.9
        {
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
