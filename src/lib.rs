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
use crate::story_system::{ActiveDialogue, Dialogue, GameState, StoryPlugin};
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
        //app.add_plugins(CommunicationsSystemPlugin);
        app.add_plugins(SpacePositionPlugin);
        app.add_plugins(GameActionsPlugin::<GameActions>::default());
        app.add_plugins(SolarSystemPlugin);
        app.add_plugins(PlayerShipPlugin);
        app.add_plugins(StoryPlugin);
        app.add_systems(Startup, startup);
        app.add_systems(Update, (fps_update,handle_input));
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
    mut app_exit: EventWriter<AppExit>,
    mut game_state: ResMut<GameState>,
    game_actions: Res<ActionState<GameActions>>,
    ship_position: Single<&SpacePosition,With<MyShip>>,
    query_dialogues: Query<(&Dialogue, &SpacePosition)>,
) {
    if game_actions.just_pressed(GameActions::Exit) {
        app_exit.send(AppExit::Success);
    }
    if game_actions.just_pressed(GameActions::Hail) {
        
        // find closet Dialogue to ship.
        let mut dialogues = query_dialogues
            .iter()
            .map(|(dialogue,pos)| (dialogue,pos.0.distance(ship_position.0)))
            .collect::<Vec<_>>();
        dialogues.sort_by(|(_,a),(_,b)| a.partial_cmp(b).unwrap());
        
        if !dialogues.is_empty() {
            let new_dialogue = dialogues.first().unwrap();
            let new_dialogue = new_dialogue.0;

            // set nearest dialogue to active. Remove old one.
            active_dialogue.dialogue = Some(new_dialogue.clone());
            active_dialogue.node_id = "start".to_string();
            println!("{:?}",new_dialogue);
        } else {
            active_dialogue.dialogue = None;
        }
        
    }
    if game_actions.just_pressed(GameActions::Choose1) {

    }
    if game_actions.just_pressed(GameActions::Choose2) {

    }
    if game_actions.just_pressed(GameActions::Choose3) {
 
    }
    if game_actions.just_pressed(GameActions::Choose4) {

    }
}