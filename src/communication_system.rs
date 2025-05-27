use crate::story_system::{ActiveDialogue, GameFlags};
use bevy::prelude::*;
use bevy::text::TextBounds;

pub struct CommunicationsSystemPlugin;

impl Plugin for CommunicationsSystemPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_window);
    }
}

fn update_window(
    mut cmd: Commands,
    asset_server: ResMut<AssetServer>,
    active_dialogue: ResMut<ActiveDialogue>,
    flags: ResMut<GameFlags>,
    mut comms_window: Query<(Entity,&mut Text2d), With<CommsWindow>>,
) {
    if !comms_window.is_empty() {
        let entity = comms_window.single_mut().0;
        cmd.entity(entity).despawn_descendants();
        cmd.entity(entity).despawn_recursive();
    }
    if let Some(message) = active_dialogue.get_message(&flags) {
        if comms_window.is_empty() {
            let image = asset_server.load("9_patch_scifi.png");
            let entity = cmd
                .spawn((
                    CommsWindow,
                    Sprite {
                        image: image.clone(),
                        custom_size: Some(Vec2::new(400.0, 200.0)),
                        image_mode: SpriteImageMode::Sliced(TextureSlicer {
                            border: BorderRect::square(32.0),
                            ..default()
                        }),
                        ..default()
                    },
                    Transform::from_translation(Vec3::new(100.0, 100.0, 100.0)),
                ))
                .with_child((
                    Transform::from_xyz(0.0, 0.0, 10.0),
                    TextLayout::new(JustifyText::Left, LineBreak::WordBoundary),
                    TextBounds::from(Vec2::new(400.0, 200.0)),
                    Text2d(message.to_string()),
                ))
                .id();
            if let Some(choices) = active_dialogue.get_choices(&flags) {
                let choices_message = choices
                    .iter()
                    .map(|c| c.text.as_ref())
                    .collect::<Vec<&str>>()
                    .join("\n");
                cmd.entity(entity)
                    .with_child((
                        Transform::from_xyz(0.0, -200.0, 10.0),
                        Sprite {
                            image,
                            custom_size: Some(Vec2::new(400.0, 200.0)),
                            image_mode: SpriteImageMode::Sliced(TextureSlicer {
                                border: BorderRect::square(32.0),
                                ..default()
                            }),
                            ..default()
                        },
                    ))
                    .with_child((
                        Transform::from_xyz(0.0, -200.0, 20.0),
                        TextLayout::new(JustifyText::Left, LineBreak::WordBoundary),
                        TextBounds::from(Vec2::new(400.0, 200.0)),
                        Text2d(choices_message),
                    ));
            }
        } 
    } 
}

#[derive(Component)]
pub struct CommsWindow;
