
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

