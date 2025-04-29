use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashSet;
use std::fmt::Debug;
use crate::solar_system::{SolarBody};
use crate::space_position::SpacePosition;

pub struct StoryPlugin;
impl Plugin for StoryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameState>();
        app.init_resource::<GameFlags>();
        app.init_resource::<ActiveDialogue>();
        app.add_systems(Startup,setup);
        app.add_systems(Update,handle_story);
    }
}

#[derive(Component)]
pub struct StoryDebug;

fn setup(
    mut commands: Commands,
) {
    commands.spawn((
        StoryDebug,
        Text2d("".to_string()),
        Transform::from_xyz(-128.0, -128.0, 1.0),
    ));
}

fn handle_story(
    mut game_state: ResMut<GameState>,
    mut flags: ResMut<GameFlags>,
    mut active_dialogue: ResMut<ActiveDialogue>,
    mut text: Single<&mut Text2d,With<StoryDebug>>,
) {
    if let Some(dialogue) = &active_dialogue.dialogue {
        if let Some(msg) = dialogue.get_text(&active_dialogue.node_id, &flags) {
            text.0 = msg.text.clone();
        }
    }
}

#[derive(Deserialize, Debug, Component, Clone)]
pub struct Dialogue {
    pub entry: String,
    pub nodes: Vec<Node>,
}
#[derive(Deserialize, Debug, Clone)]
pub struct Node {
    pub id: String,
    pub texts: Vec<Text>,
    pub choices: Vec<Choice>,
    pub on_enter: Option<Vec<String>>,
}
#[derive(Deserialize, Debug, Clone)]
pub struct Text {
    pub condition: Option<String>,
    pub text: String,
}
#[derive(Deserialize, Debug, Clone)]
pub struct Choice {
    pub text: String,
    pub next: String,
    pub condition: Option<String>,
    pub actions: Option<Vec<String>>,
}
#[derive(Debug, Resource)]
pub struct GameState {
    pub hail: bool,
}
#[derive(Resource)]
pub struct ActiveDialogue {
    pub dialogue: Option<Dialogue>,
    pub node_id: String,
}

impl FromWorld for ActiveDialogue {
    fn from_world(world: &mut World) -> Self {
        ActiveDialogue {
            dialogue: None,
            node_id: "".to_string(),
        }
    }
}

impl FromWorld for GameState {
    fn from_world(_world: &mut World) -> Self {
        GameState {
            hail: false,       
        }
    }
}

impl Dialogue {
    // Get the text for a node based on the current game state
    pub fn get_text(&self, node_id: &str, flags: &GameFlags) -> Option<&Text> {
        self.nodes
            .iter()
            .find(|n| n.id == node_id)
            .and_then(|node| {
                node.texts
                    .iter()
                    .find(|t| {
                        t.condition
                            .as_ref()
                            .map_or(true, |c| flags.contains(c))
                    })
                    .or_else(|| node.texts.last())
            })
    }

    // Get the available choices for a node based on the current game state
    pub fn get_choices(&self, node_id: &str, flags: &GameFlags) -> Vec<Choice> {
        self.nodes
            .iter()
            .find(|n| n.id == node_id)
            .map(|node| {
                node.choices
                    .iter()
                    .filter(|c| {
                        c.condition
                            .as_ref()
                            .map_or(true, |cond| flags.contains(cond))
                    }).map(|c| c.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    // Apply on_enter actions for a node, modifying the game state
    fn apply_on_enter(&self, node_id: &str, flags: &mut GameFlags) {
        if let Some(node) = self.nodes.iter().find(|n| n.id == node_id) {
            if let Some(actions) = &node.on_enter {
                for action in actions {
                    perform_action(action, flags);
                }
            }
        }
    }

    fn clone_self(&self) -> Self {
        self.clone()
    }
}

#[derive(Resource)]
pub struct GameFlags(HashSet<String>);

impl Debug for GameFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }   
}

impl Default for GameFlags {
    fn default() -> Self {
        Self(HashSet::new())
    }
}

impl GameFlags {
    pub fn set(&mut self, flag: &str) {
        self.0.insert(flag.to_string());
    }
    pub fn remove(&mut self, flag: &str) {
        self.0.remove(flag);
    }
    pub fn contains(&self, flag: &str) -> bool {
        self.0.contains(flag)
    }
}

// Perform an action to modify the game state
pub fn perform_action(action: &str, state: &mut GameFlags) {
    let parts: Vec<&str> = action.split(':').collect();
    if parts.len() == 2 {
        match parts[0] {
            "set_flag" => {
                state.set(parts[1]);
            }
            "remove_flag" => {
                state.remove(parts[1]);
            }
            _ => println!("Unknown action: {}", action),
        }
    } else {
        println!("Invalid action format: {}", action);
    }
}
