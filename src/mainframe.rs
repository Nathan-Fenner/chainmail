use bevy::{platform::collections::HashSet, prelude::*};

use crate::{
    common::Common,
    interactible::{Activated, Interactible},
    level::{LevelName, LevelTag},
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

#[derive(Resource)]
pub struct RememberedMainframes {
    remembered: HashSet<(LevelName, IVec2)>,
}

#[derive(Component)]
#[require(
    Interactible = Interactible::radius(1.9).with_priority(2).with_dot_offset(Vec3::Y * 1.2),
    PointLight = mainframe_point_light(),
)]
pub struct Mainframe {
    /// Whether the player has activate the mainframe.
    pub active: bool,
    pub has_charge: bool,
    pub location: IVec2,
}

pub struct MainframePlugin;

impl Plugin for MainframePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(RememberedMainframes {
            remembered: HashSet::new(),
        });
        app.add_systems(
            Update,
            (
                set_mainframe_icon_system.after(crate::electricity::compute_charge_system),
                activate_computer_system,
                remember_compute_system,
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

fn remember_compute_system(
    mut remember: ResMut<RememberedMainframes>,
    mut mainframes: Query<(&mut Mainframe, &LevelTag)>,
) {
    for (mut mainframe, level) in mainframes.iter_mut() {
        let key = (level.level.clone(), mainframe.location);
        if mainframe.active {
            if !remember.remembered.contains(&key) {
                remember.remembered.insert(key);
            }
        } else if remember.remembered.contains(&key) {
            mainframe.active = true;
        }
    }
}

#[derive(Component)]
pub struct HasGlow;

#[derive(Component)]
pub struct WinMainframe;

pub fn recolor_computer(
    mut commands: Commands,
    mut mainframe: Query<
        (Entity, &Mainframe, &mut PointLight, Option<&WinMainframe>),
        Changed<Mainframe>,
    >,
    common: Res<Common>,
    has_glow: Query<&HasGlow>,
) {
    for (entity, mainframe, mut light, is_win) in mainframe.iter_mut() {
        let is_win = is_win.is_some();
        if mainframe.active {
            if !has_glow.contains(entity) {
                // Spawn a glow for the computer
                commands.entity(entity).insert(HasGlow);
                commands.entity(entity).with_child((
                    Mesh3d(common.mesh_cube.clone()),
                    MeshMaterial3d(if is_win {
                        common.material_you_win.clone()
                    } else {
                        common.material_active.clone()
                    }),
                    Transform::from_translation(Vec3::Y * 1.4)
                        .with_scale(Vec3::new(1.0, 0.1, 1.2))
                        .looking_to(Vec3::Z + Vec3::Y * 0.7, Vec3::Y),
                ));
            }
            light.intensity = light_consts::lux::RAW_SUNLIGHT;
        } else {
            light.intensity = 0.0;
        }
    }
}
