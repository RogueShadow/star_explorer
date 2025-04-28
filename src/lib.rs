mod background_stars;
mod solar_system;
mod space_position;
mod navigation_system;
#[macro_use]
mod input_actions;
mod communication_system;
mod player_ship;
mod notification_system;
mod story_system;

use std::collections::HashMap;
use background_stars::BackgroundStarsPlugin;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::window::{PresentMode, WindowResolution};
use serde::{Deserialize, Serialize};
use std::fs;
use bevy::sprite::Anchor;
use space_position::*;
use solar_system::*;
use communication_system::*;
use input_actions::{GameActionsPlugin};
use navigation_system::*;
use player_ship::*;
use crate::input_actions::ActionState;
use crate::story_system::StoryPlugin;

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
        app.add_systems(Update, (
            fps_update,
        ));
    }
}

#[derive(Component)]
struct StatusText;

fn startup(
    mut commands: Commands,
    mut clear: ResMut<ClearColor>,
    asset_server: ResMut<AssetServer>,
    mut gizmos_config: ResMut<GizmoConfigStore>,
    mut ambient: ResMut<AmbientLight>,
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
    ]);

    commands.spawn((
        StatusText,
        Text2d("FPS".to_string()),
        Transform::from_xyz(-(1920.0 / 2.0) + 20.0, (1280.0 / 2.0), 0.0),
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
    
    load_solar_system(&mut commands, &asset_server, Vec2::ZERO, &solar_system, &mut meshes, &mut materials, &mut 0.0);

}

#[derive(Eq,PartialEq,Hash,Copy, Clone)]
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
}