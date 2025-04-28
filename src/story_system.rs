use std::collections::HashSet;
use bevy::prelude::*;
use serde::Deserialize;

pub struct StoryPlugin;
impl Plugin for StoryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameState>();
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
#[derive(Deserialize, Debug, Resource)]
pub struct GameState {
    pub flags: HashSet<String>,
    pub active_node: Option<String>,
}
impl FromWorld for GameState {
    fn from_world(_world: &mut World) -> Self {
        GameState {
            flags: HashSet::new(),
            active_node: None,
        }
    }
}

impl GameState {
    fn new() -> Self {
        GameState {
            flags: HashSet::new(),
            active_node: None,
        }
    }
}

impl Dialogue {
    // Get the text for a node based on the current game state
    pub fn get_text(&self, node_id: &str, state: &GameState) -> Option<&Text> {
        self.nodes
            .iter()
            .find(|n| n.id == node_id)
            .and_then(|node| {
                node.texts
                    .iter()
                    .find(|t| {
                        t.condition
                            .as_ref()
                            .map_or(true, |c| check_condition(c, state))
                    })
                    .or_else(|| node.texts.last())
            })
    }

    // Get the available choices for a node based on the current game state
    pub fn get_choices(&self, node_id: &str, state: &GameState) -> Vec<&Choice> {
        self.nodes
            .iter()
            .find(|n| n.id == node_id)
            .map(|node| {
                node.choices
                    .iter()
                    .filter(|c| {
                        c.condition
                            .as_ref()
                            .map_or(true, |cond| check_condition(cond, state))
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    // Apply on_enter actions for a node, modifying the game state
    fn apply_on_enter(&self, node_id: &str, state: &mut GameState) {
        if let Some(node) = self.nodes.iter().find(|n| n.id == node_id) {
            if let Some(actions) = &node.on_enter {
                for action in actions {
                    perform_action(action, state);
                }
            }
        }
    }
}

// Check if a condition is true based on the current state
pub fn check_condition(condition: &str, state: &GameState) -> bool {
    if condition.starts_with('!') {
        let flag = &condition[1..];
        !state.flags.contains(flag)
    } else {
        state.flags.contains(condition)
    }
}

// Perform an action to modify the game state
pub fn perform_action(action: &str, state: &mut GameState) {
    let parts: Vec<&str> = action.split(':').collect();
    if parts.len() == 2 {
        match parts[0] {
            "set_flag" => {
                state.flags.insert(parts[1].to_string());
            }
            "remove_flag" => {
                state.flags.remove(parts[1]);
            }
            _ => println!("Unknown action: {}", action),
        }
    } else {
        println!("Invalid action format: {}", action);
    }
}

