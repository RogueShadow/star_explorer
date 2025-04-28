use crate::input_actions::ActionState;
use crate::player_ship::MyShip;
use crate::solar_system::SolarBody;
use crate::space_position::SpacePosition;
use crate::story_system::{Dialogue, GameState};
use crate::GameActions;
use bevy::app::AppExit;
use bevy::asset::Assets;
use bevy::color::Color;
use bevy::prelude::*;
use bevy::sprite::Anchor;

pub struct CommunicationsSystemPlugin;

impl Plugin for CommunicationsSystemPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CommunicationWindow::default());
        app.add_systems(Startup, setup_window);
        app.add_systems(Update, update_window);
        app.add_systems(Update, communications);
    }
}

#[derive(Component)]
pub struct CommUI;

#[derive(Component)]
pub struct CommMessages;

#[derive(Resource)]
pub struct CommunicationWindow {
    pub title: String,
    pub messages: String,
    pub width: f32,
    pub height: f32,
    pub border_width: f32,
    pub text_color: Color,
    pub background_color: Color,
    pub border_color: Color,
    pub show: bool,
    pub comm_range: f32,
}
impl CommunicationWindow {
    pub fn send(&mut self, sender: &str, message: &str) {
        let new_line = &format!("{}: {}", sender, message);

        let lines = self.messages.split('\n').count();
        if lines < 7 {
            self.messages.push('\n');
            self.messages.push_str(new_line);
        } else {
            if let Some((_, last)) = self.messages.split_once('\n') {
                self.messages = last.to_owned();
            }
            self.messages.push('\n');
            self.messages.push_str(new_line);
        }
    }
    pub fn clear(&mut self) {
        self.messages = String::new();
    }
}
impl Default for CommunicationWindow {
    fn default() -> Self {
        CommunicationWindow {
            title: "Communications".to_string(),
            messages: "".to_string(),
            width: 800.0,
            height: 200.0,
            border_width: 1.0,
            text_color: Color::srgb(0.9, 0.9, 0.85),
            background_color: Color::srgb(0.1, 0.15, 0.2),
            border_color: Color::srgb(0.0, 0.9, 0.9),
            show: true,
            comm_range: 128.0,
        }
    }
}

fn setup_window(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    comm_ui: ResMut<CommunicationWindow>,
    window: Single<&Window>,
) {
    let total_ui_width = comm_ui.width + comm_ui.border_width + comm_ui.border_width;
    let total_ui_height = comm_ui.height + comm_ui.border_width + comm_ui.border_width;

    let x_position = -window.width() * 0.5 + total_ui_width * 0.5;
    let y_position = -window.height() * 0.5 + total_ui_height * 0.5;

    let bg_color = materials.add(comm_ui.background_color);
    let border_color = materials.add(comm_ui.border_color);

    commands
        .spawn((
            CommUI,
            Visibility::Hidden,
            Mesh2d(meshes.add(Rectangle::new(comm_ui.width, comm_ui.height))),
            MeshMaterial2d(bg_color),
            Transform::from_xyz(x_position, y_position, 100.0),
        ))
        .with_children(|child| {
            // add background
            child.spawn((
                Mesh2d(meshes.add(Rectangle::new(total_ui_width, total_ui_height))),
                MeshMaterial2d(border_color.clone()),
                Transform::from_xyz(0.0, 0.0, -1.0),
            ));
            // add title message.
            child.spawn((
                Text2d(comm_ui.title.clone()),
                TextColor(comm_ui.text_color),
                Transform::from_xyz(-comm_ui.width * 0.5, comm_ui.height * 0.5, 1.0),
                Anchor::TopLeft,
            ));
            // add a separator from the title areax.
            child.spawn((
                Mesh2d(meshes.add(Rectangle::new(total_ui_width, comm_ui.border_width))),
                MeshMaterial2d(border_color.clone()),
                Transform::from_xyz(0.0, comm_ui.height * 0.5 - 26.0, 1.0),
            ));
            // add comm message.
            child.spawn((
                CommMessages,
                Text2d(comm_ui.messages.clone()),
                TextColor(comm_ui.text_color),
                Transform::from_xyz(-comm_ui.width * 0.5, -comm_ui.height * 0.5, 1.0),
                Anchor::BottomLeft,
            ));
        });
}

fn update_window(
    comm_ui: Res<CommunicationWindow>,
    mut message: Query<&mut Text2d, With<CommMessages>>,
) {
    let mut text = message.single_mut();
    text.0 = comm_ui.messages.clone();
}

fn communications(
    mut commands: Commands,
    actions: Res<ActionState<GameActions>>,
    mut app_exit: EventWriter<AppExit>,
    mut comm_ui: ResMut<CommunicationWindow>,
    comm_window: Query<Entity, With<CommUI>>,
    comm_bodies: Query<(&SolarBody, &Dialogue, &SpacePosition), With<Dialogue>>,
    ship: Query<&SpacePosition, With<MyShip>>,
    state: ResMut<GameState>,
) {
    if actions.pressed(GameActions::Exit) {
        app_exit.send(AppExit::Success);
    }
    if actions.just_pressed(GameActions::ToggleCommsWindow) {
        comm_ui.show = !comm_ui.show;
    }
    if comm_ui.show {
        if let Ok(e) = comm_window.get_single() {
            commands.entity(e).insert(Visibility::Visible);
        }
    } else {
        if let Ok(e) = comm_window.get_single() {
            commands.entity(e).insert(Visibility::Hidden);
        }
    }
    if actions.just_pressed(GameActions::Hail) {
        let ship_position = ship.single().0;
        let mut bodies = comm_bodies
            .iter()
            .map(|(body, comm_messages, SpacePosition(body_pos))| {
                let distance = ship_position.distance(*body_pos);
                (distance, body.name.clone(), comm_messages)
            })
            .collect::<Vec<_>>();
        bodies.sort_by(|(d1, _, _), (d2, _, _)| d1.total_cmp(d2));
        if !bodies.is_empty() {
            if let Some((_, name, dialogue)) = bodies.first() {
                let current_node = if let Some(node_id) = state.active_node.clone() {
                    node_id
                } else {
                    "start".to_string()
                };
                let text = dialogue
                    .get_text(&current_node, &*state)
                    .expect("Couldn't get state");
                let choices = dialogue.get_choices(&current_node, &*state);
                comm_ui.send(name.as_str(), &format!("{:?}", text.text));
                let labels = vec!["a", "b", "c", "d", "e"];
                for (index, choice) in choices.iter().enumerate() {
                    comm_ui.send(labels.get(index).expect("char"), choice.text.as_str());
                }
            }
        }
    }
}
