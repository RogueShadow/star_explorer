use bevy::prelude::*;
use rand::Rng;
use std::cmp::Ordering::{Equal, Greater, Less};

#[derive(Component)]
pub struct BackgroundStar;

#[derive(Resource)]
pub struct BackgroundStarConfig {
    pub speed: f32,
    pub direction: f32,
    pub number: u32,
    pub layer: i32,
}
impl Default for BackgroundStarConfig {
    fn default() -> Self {
        Self {
            speed: 0.0,
            direction: 0.0,
            number: 200,
            layer: -1000,
        }
    }
}
impl BackgroundStarConfig {
    pub fn stars(stars: u32) -> Self {
        Self {
            number: stars,
            ..BackgroundStarConfig::default()
        }
    }
}

pub struct BackgroundStarsPlugin {
    pub number: u32,
}
impl BackgroundStarsPlugin {
    pub fn new(number: u32) -> Self {
        Self { number }
    }
}

impl Plugin for BackgroundStarsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BackgroundStarConfig::stars(self.number));
        app.add_systems(PostStartup, populate_stars);
        app.add_systems(Update, move_stars);
    }
}

fn get_star_edge_position(size: &Vec2, direction: &Vec2) -> Vec2 {
    let mut rng = rand::thread_rng();
    use std::f64::consts::{FRAC_PI_2, PI};

    let randx = rng.gen_range(0.0..size.x) - (size.x / 2.0);
    let randy = rng.gen_range(0.0..size.y) - (size.y / 2.0);
    let (minx, maxx) = (size.x / 2.0, -size.x / 2.0);
    let (miny, maxy) = (size.y / 2.0, -size.y / 2.0);

    let angle = direction.to_angle() as f64 + PI;
    let pos = match (direction.x.partial_cmp(&0.0), direction.y.partial_cmp(&0.0)) {
        (Some(x), Some(y)) => {
            use std::cmp::Ordering::*;
            match (x, y) {
                (Less, Less) => {
                    if rng.gen_bool(1.0 - (angle / FRAC_PI_2)) {
                        Vec2::new(minx, randy)
                    } else {
                        Vec2::new(randx, miny)
                    }
                }
                (Greater, Less) => {
                    if rng.gen_bool((angle - FRAC_PI_2) / FRAC_PI_2) {
                        Vec2::new(maxx, randy)
                    } else {
                        Vec2::new(randx, miny)
                    }
                }
                (Greater, Greater) => {
                    if rng.gen_bool((angle - PI) / FRAC_PI_2) {
                        Vec2::new(randx, maxy)
                    } else {
                        Vec2::new(maxx, randy)
                    }
                }
                (Less, Greater) => {
                    if rng.gen_bool((angle - (PI + FRAC_PI_2)) / FRAC_PI_2) {
                        Vec2::new(minx, randy)
                    } else {
                        Vec2::new(randx, maxy)
                    }
                }
                (Greater, Equal) => Vec2::new(maxx, randy),
                (Less, Equal) => Vec2::new(minx, randy),
                (Equal, Greater) => Vec2::new(randx, maxy),
                (Equal, Less) => Vec2::new(randx, miny),
                (Equal, Equal) => Vec2::new(randx, randy),
            }
        }
        _ => Vec2::new(randx, randy),
    };
    pos
}

fn get_random_background_star(
    size: &Vec2,
    direction: &Vec2,
) -> (BackgroundStar, Sprite, Transform) {
    let mut rng = rand::thread_rng();
    let color = rng.gen_range(0.25..0.75);
    let star_size = rng.gen_range(0.75..3.0);
    let pos = get_star_edge_position(&size, &direction);
    (
        BackgroundStar,
        Sprite::from_color(Color::srgb(color, color, color + 0.25), Vec2::splat(1.0)),
        Transform::from_xyz(pos.x, pos.y, -100.0).with_scale(Vec3::splat(star_size)),
    )
}

fn populate_stars(
    mut commands: Commands,
    window: Single<&Window>,
    config: Res<BackgroundStarConfig>,
) {
    let size = window.resolution.size();
    for _ in 0..config.number {
        commands.spawn(get_random_background_star(&size, &Vec2::ZERO));
    }
}

fn move_stars(
    window: Single<&Window>,
    config: ResMut<BackgroundStarConfig>,
    mut stars_query: Query<(Entity, &mut Transform), With<BackgroundStar>>,
    mut commands: Commands,
    time: Res<Time>,
) {
    let delta_speed = time.delta_secs() * 75.0;
    let dir = Vec2::new(
        config.speed * config.direction.cos(),
        config.speed * config.direction.sin(),
    );

    let half_size = window.resolution.size() / 2.0;

    let count = stars_query.iter().count() as u32;

    for (entity, mut star) in stars_query.iter_mut() {
        star.translation.x += dir.x * star.scale.x * delta_speed;
        star.translation.y += dir.y * star.scale.y * delta_speed;
        if star.translation.x.abs() > half_size.x || star.translation.y.abs() > half_size.y {
            match config.number.cmp(&count) {
                Less => {
                    commands.entity(entity).despawn_recursive();
                }
                Greater => {
                    let transform = get_star_edge_position(&window.resolution.size(), &dir);
                    star.translation = transform.extend(config.layer as f32);
                    commands.spawn(get_random_background_star(&window.resolution.size(), &dir));
                }
                Equal => {
                    let transform = get_star_edge_position(&window.resolution.size(), &dir);
                    star.translation = transform.extend(config.layer as f32);
                }
            }
        }
    }

    if count == 0 && config.number > 0 {
        commands.spawn(get_random_background_star(&window.resolution.size(), &dir));
    }
}
