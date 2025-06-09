use bevy::prelude::*;

use crate::common::Common;

pub struct IntroPlugin;

impl Plugin for IntroPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, advance_intro_system);
    }
}

#[derive(Component)]
struct IntroNode {
    progress: i32,
}

#[derive(Component)]
struct UICamera;

fn advance_intro_system(
    mut commands: Commands,
    mut intro: Query<(Entity, &mut ImageNode, &mut IntroNode)>,
    keys: Res<ButtonInput<KeyCode>>,
    common: Res<Common>,
    camera: Query<Entity, With<UICamera>>,
) {
    if !keys.just_pressed(KeyCode::Enter) {
        return;
    }
    for (entity, mut intro_node, mut progress) in intro.iter_mut() {
        progress.progress += 1;
        if progress.progress == 1 {
            intro_node.image = common.image_fwd.clone();
        } else if progress.progress == 2 {
            commands.entity(entity).despawn();
            for camera in camera.iter() {
                commands.entity(camera).despawn();
            }
        }
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // UI Camera
    commands.spawn((
        Camera2d,
        Camera {
            order: 1,
            ..default()
        },
        UICamera,
    ));

    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,

            ..default()
        })
        .with_children(|children| {
            children.spawn((
                ImageNode {
                    image: asset_server.load("email_cover.png"),

                    ..default()
                },
                IntroNode { progress: 0 },
            ));
        });
    /*
    let image = asset_server.load("textures/fantasy_ui_borders/panel-border-010.png");

    let slicer = TextureSlicer {
        border: BorderRect::all(22.0),
        center_scale_mode: SliceScaleMode::Stretch,
        sides_scale_mode: SliceScaleMode::Stretch,
        max_corner_scale: 1.0,
    };
    // ui camera
    commands.spawn(Camera2d);
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        })
        .with_children(|parent| {
            for [w, h] in [[150.0, 150.0], [300.0, 150.0], [150.0, 300.0]] {
                parent
                    .spawn((
                        Button,
                        ImageNode {
                            image: image.clone(),
                            image_mode: NodeImageMode::Sliced(slicer.clone()),
                            ..default()
                        },
                        Node {
                            width: Val::Px(w),
                            height: Val::Px(h),
                            // horizontally center child text
                            justify_content: JustifyContent::Center,
                            // vertically center child text
                            align_items: AlignItems::Center,
                            margin: UiRect::all(Val::Px(20.0)),
                            ..default()
                        },
                    ))
                    .with_child((
                        Text::new("Button"),
                        TextFont {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 33.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    ));
            }
        });
         */
}
