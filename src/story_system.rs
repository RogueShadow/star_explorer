use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashSet;
use crate::solar_system::{SolarBody};
use crate::space_position::SpacePosition;

pub struct StoryPlugin;
impl Plugin for StoryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameState>();
        app.add_systems(Startup,setup);
        app.add_systems(Update,handle_story);
    }
}

#[derive(Component)]
pub struct StoryDebug;

#[derive(Debug)]
pub struct Hail {
    pub origin: SpacePosition,
    pub range: f32,
}
impl Hail {
    pub fn new(origin: SpacePosition, range: f32) -> Self {
        Self { origin, range }
    }
}

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
    dialogues: Query<(&SolarBody,&Dialogue,&SpacePosition)>,
    mut text: Single<&mut Text2d,With<StoryDebug>>,
) {
    if let Some(hail) = game_state.hail.as_mut() {
        let mut bodies = dialogues
            .iter()
            .map(|(body, comm_messages, SpacePosition(body_pos))| {
                let distance = hail.origin.0.distance(*body_pos);
                (distance, body.name.clone(), comm_messages)
            })
            .filter(|(distance, _, _)| {
                *distance <= hail.range
            })
            .collect::<Vec<_>>();
        bodies.sort_by(|(d1, _, _), (d2, _, _)| d1.total_cmp(d2));

        if let Some((distance, body, dialogue)) = bodies.first() {
            if game_state.active_node.is_none() {
                game_state.active_node = Some(dialogue.entry.clone());
            }
            let message = dialogue.get_text(game_state.active_node.as_ref().unwrap(), &game_state);
            let choices = dialogue.get_choices(game_state.active_node.as_ref().unwrap(), &game_state);
            game_state.message = message.map(|m| m.text.clone());
            game_state.hail = None;
            game_state.active_dialogue = Some(dialogue.clone_self());

            text.0 = message.unwrap().text.clone();
            text.0.push_str("\n");
            for choice in choices.iter() {
                text.0.push_str(choice.text.as_str());
                text.0.push_str("\n");
            }
            game_state.choices = Some(choices);
            println!("{:?}",game_state.flags);
        }
    }
    if game_state.choose.is_some() && game_state.choices.is_some() {
        let dialogue = game_state.active_dialogue.clone().unwrap();
        let choices = game_state.choices.clone().unwrap();
        let choice = game_state.choose.unwrap();
        let choice = &choices[choice];
        game_state.active_node = Some(choice.next.clone());



        game_state.choose = None;
        game_state.choices = None;

        text.0 = game_state.message.clone().unwrap();
        text.0.push_str("\n");
        for choice in choices.iter() {
            text.0.push_str(choice.text.as_str());
            text.0.push_str("\n");
        }
        println!("{:?}",game_state.flags);
        
        for action in choice.clone().actions.unwrap().iter() {
            perform_action(action, &mut game_state);
        }

        game_state.message = Some(dialogue.get_text(game_state.active_node.as_ref().unwrap(), &game_state).unwrap().text.clone());

    } else {
        game_state.choose = None;
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
    pub flags: HashSet<String>,
    pub active_node: Option<String>,
    pub active_dialogue: Option<Dialogue>,
    pub hail: Option<Hail>,
    pub message: Option<String>,
    pub choices: Option<Vec<Choice>>,
    pub choose: Option<usize>,
}
impl FromWorld for GameState {
    fn from_world(_world: &mut World) -> Self {
        GameState {
            flags: HashSet::new(),
            active_node: None,
            active_dialogue: None,
            hail: None,
            message: None,
            choices: None,
            choose: None,
        }
    }
}

impl GameState {
    pub fn send_hail(&mut self, origin: SpacePosition, range: f32) {
        self.hail = Some(Hail::new(origin,range));
    }
    pub fn choose(&mut self, choice: usize) {
        self.choose = Some(choice);
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
    pub fn get_choices(&self, node_id: &str, state: &GameState) -> Vec<Choice> {
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
                    }).map(|c| c.clone())
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

    fn clone_self(&self) -> Self {
        self.clone()
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
