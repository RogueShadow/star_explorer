use crate::space_position::SpacePosition;
use crate::story_system::Dialogue;
use bevy::prelude::*;
use bevy::render::view::NoFrustumCulling;
use bevy::sprite::Anchor;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::f32::consts::TAU;
use std::fs;
use std::path::PathBuf;

pub struct SolarSystemPlugin;

impl Plugin for SolarSystemPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, update_orbitals);
    }
}

#[derive(Serialize, Deserialize)]
pub struct SolarBodyDescriptor {
    pub name: String,
    pub size: f32,
    #[serde(default)]
    pub orbit: Option<OrbitalBody>,
    #[serde(default)]
    pub tint: Option<Color>,
    #[serde(default)]
    pub image: Option<PathBuf>,
    pub children: Vec<SolarBodyDescriptor>,
}

#[derive(Component, Copy, Clone, Serialize, Deserialize)]
pub struct OrbitalBody {
    pub distance: f32, // Distance from parents SpacePosition
    #[serde(default)]
    pub period: Option<f32>, // Time in seconds to complete a revolution
    pub start: f32,    // Starting position in radians
}

#[derive(Component)]
pub struct SolarBody {
    pub name: String,
    pub radius: f32,
}
impl SolarBody {
    fn named(name: &str) -> Self {
        Self {
            name: name.to_string(),
            radius: 0.0,
        }
    }
    fn of_radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }
}

#[derive(Component)]
pub struct BodySize(pub f32);

pub fn load_solar_system(
    commands: &mut Commands,
    asset_server: &ResMut<AssetServer>,
    position: Vec2,
    config: &SolarBodyDescriptor,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    layer: &mut f32,
) -> Entity {
    let space_position = SpacePosition(position);
    let entity = commands
        .spawn((
            SolarBody::named(&config.name).of_radius(config.size),
            space_position.clone(),
            //text,
            Anchor::Custom(Vec2::new(0.0, 1.0)),
            Transform::from_xyz(0.0, 0.0, *layer),
            Visibility::Visible,
            BodySize(config.size),
            NoFrustumCulling,
        ))
        .id();
    let path = format!("assets/dialogue/{}.json", &config.name.to_lowercase());
    if fs::metadata(&path).is_ok() {
        let dialogue: Dialogue = serde_json::from_str(
            fs::read_to_string(path)
                .expect("Couldn't read a file.")
                .as_str(),
        )
        .expect("Couldn't form the Dialogue");
        println!("Loaded dialogue for {}\n {:?}", &config.name, &dialogue);
        commands.entity(entity).insert(dialogue);
    }
    match (config.tint, config.image.clone()) {
        (Some(tint), Some(pathbuf)) => {
            let image = asset_server.load(pathbuf);
            let mut sprite = Sprite::from_image(image);
            sprite.color = tint;
            sprite.custom_size = Some(Vec2::splat(config.size));
            commands.entity(entity).insert(sprite);
        }
        (None, Some(pathbuf)) => {
            let image = asset_server.load(pathbuf);
            let mut sprite = Sprite::from_image(image);
            sprite.custom_size = Some(Vec2::splat(config.size));
            commands.entity(entity).insert(sprite);
        }
        (Some(tint), None) => {
            commands.entity(entity).insert((
                Mesh2d(meshes.add(Circle::new(config.size))),
                MeshMaterial2d(materials.add(ColorMaterial::from_color(tint))),
            ));
        }
        (None, None) => {
            let mut rng = rand::thread_rng();
            commands.entity(entity).insert((
                Mesh2d(meshes.add(Circle::new(config.size))),
                MeshMaterial2d(materials.add(ColorMaterial::from_color(rand_color(&mut rng)))),
            ));
        }
    };

    if let Some(orbit) = &config.orbit {
        commands.entity(entity).insert(orbit.clone());
    }
    let mut children: Vec<Entity> = vec![];
    *layer += 1.0;
    for child in &config.children {
        children.push(load_solar_system(
            commands,
            asset_server,
            position,
            child,
            meshes,
            materials,
            layer,
        ));
    }
    commands.entity(entity).add_children(children.as_slice());
    entity
}

pub fn update_orbitals(
    parents: Query<((&SpacePosition, &BodySize), Option<&Children>)>,
    children: Query<(&OrbitalBody, &BodySize)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for ((SpacePosition(pos), BodySize(size)), maybe_children) in parents.iter() {
        if let Some(children_entities) = maybe_children {
            for &child_entity in children_entities.iter() {
                if let Ok((orbital_body, BodySize(child_size))) = children.get(child_entity) {
                    let speed = if let Some(period) = orbital_body.period {
                        TAU / period
                    } else {
                        1.0
                    };
                    let elapsed_time = (time.elapsed_secs() * speed) + orbital_body.start;
                    let new_position = Vec2::new(
                        pos.x
                            + elapsed_time.cos()
                                * (orbital_body.distance + (size + child_size) * 2.0),
                        pos.y
                            + elapsed_time.sin()
                                * (orbital_body.distance + (size + child_size) * 2.0),
                    );
                    commands
                        .entity(child_entity)
                        .insert(SpacePosition(new_position));
                }
            }
        }
    }
}

fn rand_color(rng: &mut impl Rng) -> Color {
    Color::srgb(
        rng.gen_range(0.0..=1.0),
        rng.gen_range(0.0..=1.0),
        rng.gen_range(0.0..=1.0),
    )
}
