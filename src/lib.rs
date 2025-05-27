mod background_stars;
mod navigation_system;
mod solar_system;
mod space_position;
#[macro_use]
mod input_actions;
mod communication_system;
mod notification_system;
mod player_ship;
mod story_system;

use crate::input_actions::ActionState;
use crate::story_system::{
    ActiveDialogue, Dialogue, GameFlags, GameState, StoryPlugin, perform_action, perform_actions,
};
use background_stars::BackgroundStarsPlugin;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::window::{PresentMode, WindowResolution};
use communication_system::*;
use input_actions::GameActionsPlugin;
use navigation_system::*;
use player_ship::*;
use solar_system::*;
use space_position::*;
use std::collections::HashMap;
use std::fs;

pub fn run() {
    App::new().add_plugins(StarExplorer).run();
}

fn fps_update(
    diagnostics: Res<DiagnosticsStore>,
    mut query: Single<&mut Text2d, With<StatusText>>,
) {
    if let Some(fps) = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|fps| fps.smoothed())
    {
        query.0 = format!("{fps:.0}");
    } else {
        query.0 = " N/A".to_string();
    }
}

struct StarExplorer;

impl Plugin for StarExplorer {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                present_mode: PresentMode::AutoNoVsync,
                resolution: WindowResolution::new(1600.0, 900.0),
                ..Default::default()
            }),
            ..Default::default()
        }));
        app.add_plugins(FrameTimeDiagnosticsPlugin::default());
        app.add_plugins(BackgroundStarsPlugin::new(200));
        app.add_plugins(NavigationSystemPlugin);
        app.add_plugins(CommunicationsSystemPlugin);
        app.add_plugins(SpacePositionPlugin);
        app.add_plugins(GameActionsPlugin::<GameActions>::default());
        app.add_plugins(SolarSystemPlugin);
        app.add_plugins(PlayerShipPlugin);
        app.add_plugins(StoryPlugin);
        app.add_systems(Startup, startup);
        app.add_systems(Update, (fps_update, handle_input));
    }
}

#[derive(Component)]
struct StatusText;

fn startup(
    mut commands: Commands,
    mut clear: ResMut<ClearColor>,
    asset_server: ResMut<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut action_state: ResMut<ActionState<GameActions>>,
) {
    use GameActions::*;
    use KeyCode::*;
    action_state.set_binds(binds![
        TurnLeft, ArrowLeft, KeyA;
        TurnRight, ArrowRight, KeyD;
        ThrustForward, ArrowUp, KeyW;
        ThrustReverse, ArrowDown, KeyS;
        ToggleNavMarkers, F1;
        ToggleCommsWindow, F2;
        Hail, KeyC;
        Brake, Space;
        Exit, Escape;
        Choose1, Digit1;
        Choose2, Digit2;
        Choose3, Digit3;
        Choose4, Digit4;
    ]);

    commands.spawn((
        StatusText,
        Text2d("FPS".to_string()),
        Transform::from_xyz(-(1920.0 / 2.0) + 20.0, 1280.0 / 2.0, 0.0),
        Anchor::TopLeft,
    ));

    clear.0 = Color::BLACK;

    commands.spawn(Camera2d);

    let ship = asset_server.load("ship3.png");

    commands.spawn((
        MyShip,
        Sprite::from(ship),
        Transform::from_scale(Vec3::splat(0.25)).with_translation(Vec2::ZERO.extend(10.0)),
        SpacePosition(Vec2::ZERO),
        Visibility::Visible,
    ));

    let system_string = fs::read_to_string("system_file.json").unwrap();
    let solar_system = serde_json::from_str(&system_string).unwrap();

    load_solar_system(
        &mut commands,
        &asset_server,
        Vec2::ZERO,
        &solar_system,
        &mut meshes,
        &mut materials,
        &mut 0.0,
    );
}

#[derive(Eq, PartialEq, Hash, Copy, Clone)]
pub enum GameActions {
    TurnLeft,
    TurnRight,
    ThrustForward,
    ThrustReverse,
    ToggleNavMarkers,
    Hail,
    ToggleCommsWindow,
    Brake,
    Exit,
    Choose1,
    Choose2,
    Choose3,
    Choose4,
}

fn handle_input(
    mut commands: Commands,
    mut active_dialogue: ResMut<ActiveDialogue>,
    mut flags: ResMut<GameFlags>,
    mut app_exit: EventWriter<AppExit>,
    mut game_state: ResMut<GameState>,
    game_actions: Res<ActionState<GameActions>>,
    ship_position: Single<&SpacePosition, With<MyShip>>,
    query_dialogues: Query<(Entity, &Dialogue, &SpacePosition)>,
) {
    if game_actions.just_pressed(GameActions::Exit) {
        app_exit.send(AppExit::Success);
    }
    if game_actions.just_pressed(GameActions::Hail) {
        // find closet Dialogue to ship.
        let mut dialogues = query_dialogues
            .iter()
            .map(|(e, dialogue, pos)| (e, dialogue, pos.0.distance(ship_position.0)))
            .collect::<Vec<_>>();
        dialogues.sort_by(|(_, _, a), (_, _, b)| a.partial_cmp(b).unwrap());

        if !dialogues.is_empty() {
            let new_dialogue = dialogues.first().unwrap();
            let new_entity = new_dialogue.0;
            let new_dialogue = new_dialogue.1;

            // set nearest dialogue to active.
            active_dialogue.set_active(new_dialogue, new_entity);
        } else {
            active_dialogue.clear();
        }
    }
    if game_actions.just_pressed(GameActions::ToggleCommsWindow) {
        active_dialogue.clear();
    }
    if let Some(choices) = active_dialogue.get_choices(&flags) {
        if game_actions.just_pressed(GameActions::Choose1) {
            if choices.len() > 0 {
                let choice = choices.get(0).unwrap();
                active_dialogue.set_node_id(&choice.next);
                if let Some(action) = choice.actions.as_ref() {
                    perform_actions(action, &mut flags);
                }
            }
        }
        if game_actions.just_pressed(GameActions::Choose2) {
            if choices.len() > 1 {
                let choice = choices.get(1).unwrap();
                active_dialogue.set_node_id(&choice.next);
                if let Some(action) = choice.actions.as_ref() {
                    perform_actions(action, &mut flags);
                }
            }
        }
        if game_actions.just_pressed(GameActions::Choose3) {
            if choices.len() > 2 {
                let choice = choices.get(2).unwrap();
                active_dialogue.set_node_id(&choice.next);
                if let Some(action) = choice.actions.as_ref() {
                    perform_actions(action, &mut flags);
                }
            }
        }
        if game_actions.just_pressed(GameActions::Choose4) {
            if choices.len() > 3 {
                let choice = choices.get(3).unwrap();
                active_dialogue.set_node_id(&choice.next);
                if let Some(action) = choice.actions.as_ref() {
                    perform_actions(action, &mut flags);
                }
            }
        }
    }
}

pub fn draw_patch(cmd: &mut Commands, image_handle: Handle<Image>, position: Vec2, size: Vec2) {
    cmd.spawn((
        Transform::from_xyz(position.x, position.y, 100.0),
        Sprite {
            image: image_handle,
            custom_size: Some(size),
            image_mode: SpriteImageMode::Sliced(TextureSlicer {
                border: BorderRect::square(32.0),
                ..default()
            }),
            ..default()
        },
    ));
}
