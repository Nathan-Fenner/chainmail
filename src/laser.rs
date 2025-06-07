use avian3d::prelude::*;
use bevy::{pbr::light_consts::lux::RAW_SUNLIGHT, prelude::*};

use crate::{common::Common, fog::DoesNotClearFog, level::LevelTag};

pub struct LaserPlugin;

#[derive(Component)]
pub struct Laser {
    pub direction: Vec3,
    pub beam: Option<Entity>,
}

impl Plugin for LaserPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, draw_lasers_system);
    }
}

#[derive(Component)]
struct LaserBeam;

fn draw_lasers_system(
    spatial: SpatialQuery,
    common: Res<Common>,
    mut commands: Commands,
    mut lasers: Query<(&Transform, &mut Laser, &LevelTag)>,
    mut beam: Query<&mut Transform, (With<LaserBeam>, Without<Laser>)>,
) {
    for (laser_transform, mut laser, laser_level) in lasers.iter_mut() {
        let cast = spatial.cast_ray(
            laser_transform.translation + laser.direction * 0.05,
            Dir3::try_from(laser.direction).unwrap_or(Dir3::X),
            150.,
            false,
            &SpatialQueryFilter::default(),
        );

        if let Some(cast) = cast {
            let to = laser_transform.translation + cast.distance * laser.direction;
            let from = laser_transform.translation;

            let beam_transform = Transform::from_translation((to + from) / 2.)
                .looking_at(to, Vec3::Y)
                .with_scale(Vec3::new(0.1, 0.1, to.distance(from) + 0.1));

            if let Some(beam_id) = laser.beam {
                if let Ok(mut beam) = beam.get_mut(beam_id) {
                    *beam = beam_transform;
                };
            } else {
                let beam_id = commands
                    .spawn((
                        DoesNotClearFog,
                        Mesh3d(common.mesh_cube.clone()),
                        MeshMaterial3d(common.material_laser.clone()),
                        LaserBeam,
                        PointLight {
                            range: 3.,
                            radius: 0.3,
                            intensity: RAW_SUNLIGHT,
                            color: Color::linear_rgb(1.0, 0.0, 0.3),
                            ..default()
                        },
                        beam_transform,
                        laser_level.clone(),
                    ))
                    .with_child((
                        // Light at the impact point
                        DoesNotClearFog,
                        Transform::from_translation(Vec3::new(0.0, 0.0, -0.5)),
                        PointLight {
                            range: 3.,
                            radius: 0.3,
                            intensity: RAW_SUNLIGHT,
                            color: Color::linear_rgb(1.0, 0.0, 0.3),
                            ..default()
                        },
                    ))
                    .id();

                laser.beam = Some(beam_id);
            }
        }
    }
}
