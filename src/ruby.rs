use bevy::prelude::*;

use crate::{common::Common, fog::DoesNotClearFog, level::LevelTag, player::Player};

pub struct RubyPlugin;

impl Plugin for RubyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CanZip { can_zip: false })
            .add_systems(Update, (collect_ruby_system, make_ruby_system));
    }
}

#[derive(Component)]
pub struct Ruby;

#[derive(Resource)]
pub struct CanZip {
    pub can_zip: bool,
}

pub fn collect_ruby_system(
    mut commands: Commands,
    ruby: Query<(Entity, &LevelTag, &GlobalTransform), With<Ruby>>,
    mut can_zip: ResMut<CanZip>,
    player: Query<&GlobalTransform, With<Player>>,
    common: Res<Common>,
) {
    let Ok(player) = player.single() else {
        return;
    };

    for (ruby_entity, ruby_level, ruby_transform) in ruby.iter() {
        if ruby_transform
            .translation()
            .xz()
            .distance(player.translation().xz())
            < 1.2
        {
            // Collect the ruby
            commands.entity(ruby_entity).despawn();
            can_zip.can_zip = true;

            // Spawn a tutorial above
            commands.spawn((
                ruby_level.clone(),
                Mesh3d(common.mesh_plane.clone()),
                MeshMaterial3d(common.material_tutorial_zip.clone()),
                Transform::from_translation(
                    ruby_transform.translation() + Vec3::new(0.0, 3.0, 12.0),
                )
                .looking_to(Vec3::Y + Vec3::Z * 0.2, -Vec3::Y)
                .with_scale(Vec3::splat(6.)),
                DoesNotClearFog,
            ));
        }
    }
}

#[derive(Component)]
pub struct MakeRuby;

fn make_ruby_system(
    mut commands: Commands,
    mut material: Query<&mut MeshMaterial3d<StandardMaterial>>,
    children: Query<&Children>,
    make_ruby: Query<Entity, With<MakeRuby>>,
    common: Res<Common>,
) {
    for ruby in make_ruby.iter() {
        if let Ok(mut material) = material.get_mut(ruby) {
            material.0 = common.material_ruby.clone();
        }

        if let Ok(children) = children.get(ruby) {
            for child in children.iter() {
                commands.entity(child).insert(MakeRuby);
            }
        }
    }
}
