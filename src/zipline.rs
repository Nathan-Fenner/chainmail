use avian3d::prelude::*;
use bevy::prelude::*;

use crate::{
    interactible::{Activated, Interactible},
    player::{Player, RecentVelocity},
};

const ATTACH_RADIUS: f32 = 1.9;

#[derive(Default, Component)]
#[require(Interactible = Interactible::radius(ATTACH_RADIUS))]
pub struct Zipline {
    pub nodes: Vec<Vec3>,
    pub active: Option<ZipDirection>,
    pub closest_index: usize,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum ZipDirection {
    Pos,
    Neg,
}

pub struct ZiplinePlugin;

impl Plugin for ZiplinePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                zip_prompt_system,
                pick_zipline_system,
                update_zipline_system,
                zip_on_zipline_system.after(crate::player::control_player),
            )
                .chain(),
        );
    }
}

fn zip_prompt_system(
    player: Query<&Transform, With<Player>>,
    mut zipline: Query<(&mut Transform, &mut Zipline), Without<Player>>,
) {
    let Ok(player) = player.single() else {
        return;
    };

    for (mut zipline_transform, mut zipline) in zipline.iter_mut() {
        // Move to the closest point on the zipline.

        let Some(closest) = zipline
            .nodes
            .iter()
            .enumerate()
            .min_by_key(|n| (n.1.distance(player.translation) * 100.) as i64)
        else {
            continue;
        };

        zipline_transform.translation = *closest.1;
        zipline.closest_index = closest.0;
    }
}
pub fn pick_zipline_system(
    mut zipline: Query<(&mut Zipline, &mut Activated)>,
    player_intent: Query<&RecentVelocity, With<Player>>,
) {
    for (mut zipline, mut activated) in zipline.iter_mut() {
        if activated.take_activated() {
            match zipline.active {
                None => {
                    if zipline.closest_index <= 2 {
                        zipline.active = Some(ZipDirection::Pos);
                    } else if zipline.closest_index + 2 >= zipline.nodes.len() {
                        zipline.active = Some(ZipDirection::Neg);
                    } else if zipline.nodes.len() >= 3 {
                        let node_next = zipline.nodes[zipline.closest_index + 1];
                        let node_current = zipline.nodes[zipline.closest_index];
                        let node_prev = zipline.nodes[zipline.closest_index - 1];

                        let zip_forward = (node_next - node_current).normalize();
                        let zip_backward = (node_prev - node_current).normalize();

                        let player_direction = player_intent
                            .single()
                            .map(|d| d.direction)
                            .unwrap_or(Vec3::X);
                        zipline.active = Some(
                            if player_direction.dot(zip_forward)
                                > player_direction.dot(zip_backward)
                            {
                                ZipDirection::Pos
                            } else {
                                ZipDirection::Neg
                            },
                        );
                    }
                }
                Some(_) => {
                    zipline.active = None;
                }
            }
        }
    }
}

pub fn update_zipline_system(mut zipline: Query<(&Zipline, &mut Interactible), Changed<Zipline>>) {
    for (zipline, mut interactive) in zipline.iter_mut() {
        if zipline.active.is_some() {
            interactive.priority = 5; // Do not allow disabling
            interactive.radius = 999.;
        } else {
            interactive.priority = 0;
            interactive.radius = ATTACH_RADIUS;
        }
    }
}

pub fn zip_on_zipline_system(
    mut zipline: Query<&mut Zipline>,
    mut player: Query<(&Transform, &LinearVelocity, &mut ExternalForce), With<Player>>,
) {
    let Ok((player_transform, player_velocity, mut player_force)) = player.single_mut() else {
        return;
    };
    for mut zipline in zipline.iter_mut() {
        let Some(active) = zipline.active else {
            continue;
        };

        // This zipline is active.
        // Pull the player.

        let target_index = match active {
            ZipDirection::Pos => {
                if zipline.closest_index + 1 >= zipline.nodes.len() {
                    // End of zipline
                    zipline.active = None;
                    continue;
                };
                zipline.closest_index + 1
            }
            ZipDirection::Neg => {
                if zipline.closest_index == 0 {
                    // End of zipline
                    zipline.active = None;
                    continue;
                }
                zipline.closest_index - 1
            }
        };

        let direction = zipline.nodes[target_index] + Vec3::Y - player_transform.translation;

        let direction = direction.normalize_or_zero();
        player_force.set_force(direction * 250. - **player_velocity * 10.);
    }
}
