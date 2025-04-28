use crate::background_stars::BackgroundStarConfig;
use crate::input_actions::ActionState;
use crate::space_position::SpacePosition;
use crate::GameActions;
use bevy::math::{Quat, Vec2};
use bevy::prelude::*;

pub struct PlayerShipPlugin;

impl Plugin for PlayerShipPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, move_ship);
    }
}

#[derive(Component)]
pub struct MyShip;

fn move_ship(
    mut transform_query: Query<(&mut Transform, &mut SpacePosition), With<MyShip>>,
    mut config: ResMut<BackgroundStarConfig>,
    time: Res<Time<Virtual>>,
    actions: Res<ActionState<GameActions>>,
) {
    use GameActions::*;
    let delta = time.delta_secs();
    let turn_amount = delta * 1.5f32;
    let speed_amount = delta * 0.5f32;

    if actions.pressed(TurnLeft) {
        config.direction += turn_amount;
    }
    if actions.pressed(TurnRight) {
        config.direction -= turn_amount;
    }
    if actions.pressed(ThrustForward) {
        config.speed += speed_amount;
    }
    if actions.pressed(ThrustReverse) {
        config.speed -= speed_amount;
    }
    if actions.pressed(Brake) {
        config.speed = 0.0;
    }

    config.speed = config.speed.clamp(0.0, 50.0);

    let space_movement = Vec2::new(
        config.direction.cos() * config.speed,
        config.direction.sin() * config.speed,
    );

    if let Ok((mut transform, mut space_pos)) = transform_query.get_single_mut() {
        transform.rotation = Quat::from_rotation_z(config.direction + std::f32::consts::FRAC_PI_2);
        space_pos.0 += space_movement * 200.0 * delta;
    }
}
