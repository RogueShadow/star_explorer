use crate::player_ship::MyShip;
use bevy::math::Vec2;
use bevy::prelude::*;

pub struct SpacePositionPlugin;

impl Plugin for SpacePositionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_space);
    }
}

#[derive(Component, Copy, Clone, Debug)]
pub struct SpacePosition(pub Vec2);

fn update_space(
    ship_space_position: Single<&SpacePosition, With<MyShip>>,
    mut solar_bodies: Query<(&mut GlobalTransform, &SpacePosition), Without<MyShip>>,
) {
    for (mut transform, space_position) in solar_bodies.iter_mut() {
        let new_pos = ship_space_position.0 - space_position.0;
        *transform = GlobalTransform::from(Transform::from_xyz(
            new_pos.x,
            new_pos.y,
            transform.translation().z,
        ));
    }
}
