use bevy::prelude::*;

#[derive(Component)]
pub struct Door;

#[derive(Component)]
pub struct DoorTrigger;

pub struct DoorPlugin;

impl Plugin for DoorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, detect_player_entering_door);
    }
}

fn detect_player_entering_door(
    player_query: Query<&Transform, With<crate::player::Player>>,
    trigger_query: Query<&Transform, With<DoorTrigger>>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    for trigger_transform in trigger_query.iter() {
        let distance = player_transform
            .translation
            .distance(trigger_transform.translation);
        if distance < 1.5 {
            // info!("Player entered the door zone!");
            // Insert win condition logic, scene transition, etc.
        }
    }
}
