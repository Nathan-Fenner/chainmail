use bevy::prelude::*;

use crate::electricity::PowerGrid;

#[derive(Component)]
pub struct Door {
    pub open_at: Vec3,
    pub closed_at: Vec3,
}

pub struct DoorPlugin;

impl Plugin for DoorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            door_open_system.after(crate::electricity::compute_charge_system),
        );
    }
}

fn door_open_system(mut doors: Query<(&mut Transform, &mut Door)>, power_grid: Res<PowerGrid>) {
    for (mut door_transform, door) in doors.iter_mut() {
        let grid = door_transform.translation.xz().round().as_ivec2();
        let is_open = power_grid.active.contains(&grid);

        let target_translation = if is_open {
            door.open_at
        } else {
            door.closed_at
        };

        door_transform.translation = door_transform.translation.lerp(target_translation, 0.1);
    }
}
