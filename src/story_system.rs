use bevy::prelude::*;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use bevy::sprite::Anchor;
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
        Transform::from_xyz(0.0, -300.0, 1.0),
        Anchor::BottomCenter,
    ));
}

fn handle_story(
    mut flags: ResMut<GameFlags>,
    mut active_dialogue: ResMut<ActiveDialogue>,
    mut text: Single<&mut Text2d,With<StoryDebug>>,
) {
    let mut choices = None;
    text.0.clear();
    if let Some(dialogue) = &active_dialogue.dialogue {
        dialogue.apply_on_enter(&active_dialogue.node_id(), &mut flags);
        choices = Some(dialogue.get_choices(&active_dialogue.node_id(), &mut flags));
        if let Some(msg) = dialogue.get_text(&active_dialogue.node_id(), &flags) {
            text.0.push_str(&msg.text);
            text.0.push_str("\n");
        }
    }
    active_dialogue.choices = choices;
    if let Some(choices) = &active_dialogue.choices {
        text.0.push_str("--------\n");
        for choice in choices.iter() {
            text.0.push_str(&choice.text);
            text.0.push_str("\n");
        }
    }
    text.0.push_str(format!("\n\n---flags---\n{:?}",flags.0).as_str());
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
    pub choices: Option<Vec<Choice>>,
    pub entity: Option<Entity>,
    pub node_id: HashMap<Entity, String>,
}
impl ActiveDialogue {
    pub fn node_id(&self) -> String {
        if self.entity.is_none() {
            "start".to_string()
        } else {
            self.node_id.get(&self.entity.unwrap()).unwrap_or(&"start".to_string()).clone()
        }
    }
    pub fn set_node_id(&mut self, node_id: &str) {
        self.node_id.insert(self.entity.unwrap(), node_id.to_string());
    }
}

impl FromWorld for ActiveDialogue {
    fn from_world(world: &mut World) -> Self {
        ActiveDialogue {
            dialogue: None,
            choices: None,
            entity: None,
            node_id: HashMap::new(),
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
    pub fn get_text(&self, node_id: &str, flags: &GameFlags) -> Option<&Text> {
        self.nodes.iter().find(|n| n.id == node_id).and_then(|node| {
            node.texts.iter().find(|t| flags.check(t.condition.as_deref()))
        })
    }

    pub fn get_choices(&self, node_id: &str, flags: &GameFlags) -> Vec<Choice> {
        self.nodes
            .iter()
            .find(|n| n.id == node_id)
            .map(|node| {
                node.choices
                    .iter()
                    .filter(|c| {
                        let condition = c.condition.as_deref();
                        let result = flags.check(condition);
                        result
                    })
                    .cloned()
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
    fn check(&self, condition: Option<&str>) -> bool {
        let condition_str = condition.unwrap_or("");
        if condition_str.trim().is_empty() {
            return true;
        }
        let tokens = tokenize_condition(condition_str);
        if tokens.is_empty() {
            return true;
        }
        let mut index = 0;
        self.parse_or(&tokens, &mut index)
    }

    fn parse_or(&self, tokens: &[String], index: &mut usize) -> bool {
        let mut result = self.parse_and(tokens, index);
        while *index < tokens.len() && tokens[*index] == "||" {
            *index += 1;
            let right = self.parse_and(tokens, index);
            result |= right;
        }
        result
    }

    fn parse_and(&self, tokens: &[String], index: &mut usize) -> bool {
        let mut result = self.parse_unary(tokens, index);
        while *index < tokens.len() && tokens[*index] == "&&" {
            *index += 1;
            let right = self.parse_unary(tokens, index);
            result &= right;
        }
        result
    }

    fn parse_unary(&self, tokens: &[String], index: &mut usize) -> bool {
        if *index < tokens.len() && tokens[*index] == "!" {
            *index += 1;
            !self.parse_unary(tokens, index)
        } else if *index < tokens.len() {
            let flag = &tokens[*index];
            *index += 1;
            self.0.contains(flag)
        } else {
            false
        }
    }
}

pub fn perform_actions(actions: &[String], state: &mut GameFlags) {
    for action in actions {
        perform_action(action, state);
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

fn tokenize_condition(condition: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut chars = condition.chars().peekable();

    while let Some(&ch) = chars.peek() {
        match ch {
            '!' => {
                tokens.push("!".to_string());
                chars.next();
            }
            '&' => {
                chars.next();
                if chars.peek() == Some(&'&') {
                    tokens.push("&&".to_string());
                    chars.next();
                } else {
                    let mut flag = String::new();
                    flag.push('&');
                    while let Some(&next_ch) = chars.peek() {
                        if next_ch.is_alphanumeric() || next_ch == '_' {
                            flag.push(next_ch);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    tokens.push(flag);
                }
            }
            '|' => {
                chars.next();
                if chars.peek() == Some(&'|') {
                    tokens.push("||".to_string());
                    chars.next();
                } else {
                    let mut flag = String::new();
                    flag.push('|');
                    while let Some(&next_ch) = chars.peek() {
                        if next_ch.is_alphanumeric() || next_ch == '_' {
                            flag.push(next_ch);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    tokens.push(flag);
                }
            }
            _ if ch.is_alphanumeric() || ch == '_' => {
                let mut flag = String::new();
                while let Some(&next_ch) = chars.peek() {
                    if next_ch.is_alphanumeric() || next_ch == '_' {
                        flag.push(next_ch);
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(flag);
            }
            _ => {
                chars.next(); // Skip whitespace or invalid characters
            }
        }
    }
    tokens
}