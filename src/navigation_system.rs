use crate::GameActions;
use crate::input_actions::ActionState;
use crate::solar_system::SolarBody;
use bevy::math::Vec2;
use bevy::prelude::*;
use std::collections::HashMap;

pub struct NavigationSystemPlugin;

impl Plugin for NavigationSystemPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(NavigationUI::default());
        app.add_systems(Update, point_at_nearby_bodies);
    }
}

#[derive(Resource, Default)]
pub struct NavigationUI {
    pub show: bool,
}

#[derive(Component)]
pub struct NavMarker(Entity);

fn point_at_nearby_bodies(
    bodies_query: Query<(Entity, &GlobalTransform, &SolarBody)>,
    window: Single<&Window>,
    mut commands: Commands,
    mut nav_markers: Query<(Entity, &mut Transform, &NavMarker)>,
    actions: Res<ActionState<GameActions>>,
    mut nav_ui: ResMut<NavigationUI>,
    asset_server: Res<AssetServer>,
) {
    if actions.just_pressed(GameActions::ToggleNavMarkers) {
        nav_ui.show = !nav_ui.show;
    }

    if nav_ui.show {
        for (nav_entity, _, _) in nav_markers.iter_mut() {
            commands.entity(nav_entity).despawn();
        }
        return;
    }
    let screen = window.size();
    let half_width = screen.x * 0.5;
    let half_height = screen.y * 0.5;

    let mut used_markers: HashMap<Entity, Entity> = HashMap::new();

    for (entity, trans, body) in bodies_query.iter() {
        let position = trans.translation().xy();
        if position.x.abs() > half_width || position.y.abs() > half_height {
            let direction = position.normalize_or_zero();
            if direction.is_finite() {
                let mut edge_point =
                    calculate_screen_edge_point(direction, half_width, half_height);
                let name = &body.name;
                let half_width = name.len() as f32 * 6.5;
                edge_point.y -= direction.y.signum() * 9.0;
                edge_point.x -= direction.x.signum() * half_width;

                let mut marker_found = false;
                for (nav_entity, mut nav_transform, nav_marker) in nav_markers.iter_mut() {
                    if nav_marker.0 == entity {
                        marker_found = true;
                        *nav_transform = Transform::from_translation(edge_point.extend(10.0));
                        used_markers.insert(entity, nav_entity);
                        break;
                    }
                }
                if !marker_found {
                    commands.spawn((
                        Text2d(name.to_owned()),
                        Transform::from_translation(edge_point.extend(10.0)),
                        NavMarker(entity),
                        TextFont::from_font(asset_server.load("fonts/FiraSans-Regular.ttf")),
                    ));
                }
            }
        }
    }
    for (nav_entity, _, nav_marker) in nav_markers.iter() {
        if !used_markers.contains_key(&nav_marker.0) {
            commands.entity(nav_entity).despawn();
        }
    }
}

fn calculate_screen_edge_point(direction: Vec2, half_width: f32, half_height: f32) -> Vec2 {
    let dir_x = direction.x;
    let dir_y = direction.y;
    let intersects_horizontal = dir_x.abs() * half_height > dir_y.abs() * half_width;

    if intersects_horizontal {
        let x = half_width.copysign(dir_x);
        Vec2::new(x, x * dir_y / dir_x)
    } else {
        let y = half_height.copysign(dir_y);
        Vec2::new(y * dir_x / dir_y, y)
    }
}
