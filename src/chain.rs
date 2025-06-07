use bevy::prelude::*;

#[derive(Component)]
pub struct ChainLink(pub Entity, pub Entity);

pub struct ChainPlugin;

impl Plugin for ChainPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, draw_chains_system);
    }
}

fn draw_chains_system(
    transform: Query<&Transform, Without<ChainLink>>,
    mut links: Query<(&mut Transform, &ChainLink)>,
) {
    for (mut link_transform, link) in links.iter_mut() {
        let Ok(a) = transform.get(link.0) else {
            continue;
        };
        let Ok(b) = transform.get(link.1) else {
            continue;
        };
        let a = a.translation;
        let b = b.translation;

        *link_transform = Transform::from_translation((a + b) / 2.0)
            .looking_at(a, Vec3::Y)
            .with_scale(Vec3::new(0.2, 0.2, a.distance(b) + 0.2));
    }
}
