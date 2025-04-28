use bevy::input::ButtonInput;
use bevy::prelude::*;
use std::collections::HashMap;

pub struct GameActionsPlugin<T: Eq + std::hash::Hash + Clone + Send + Sync + 'static> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Eq + std::hash::Hash + Clone + Send + Sync + 'static> Default for GameActionsPlugin<T> {
    fn default() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T: Eq + std::hash::Hash + Clone + Send + Sync + 'static> Plugin for GameActionsPlugin<T> {
    fn build(&self, app: &mut App) {
        app.insert_resource(ActionState::<T> {
            pressed: vec![],
            just_pressed: vec![],
            just_released: vec![],
            map: HashMap::default(),
        });
        app.add_systems(PreUpdate, get_actions_from_input::<T>);
    }
}

#[derive(Resource)]
pub struct ActionState<T> {
    pressed: Vec<T>,
    just_pressed: Vec<T>,
    just_released: Vec<T>,
    map: HashMap<T, Vec<KeyCode>>,
}
#[allow(dead_code)]
impl<T: PartialEq + Send + Sync + 'static + Clone> ActionState<T> {
    pub fn set_binds(&mut self, binds: HashMap<T, Vec<KeyCode>>) {
        self.map = binds;
    }
    pub fn pressed(&self, action: T) -> bool {
        self.pressed.contains(&action)
    }
    pub fn just_pressed(&self, action: T) -> bool {
        self.just_pressed.contains(&action)
    }
    pub fn just_released(&self, action: T) -> bool {
        self.just_released.contains(&action)
    }
}

pub fn get_actions_from_input<T: Eq + std::hash::Hash + Clone + Send + Sync + 'static>(
    key: Res<ButtonInput<KeyCode>>,
    mut actions: ResMut<ActionState<T>>,
) {
    actions.pressed.clear();
    actions.just_pressed.clear();
    actions.just_pressed.clear();

    let map = actions.map.clone();
    for (action, keys) in map.iter() {
        if key.any_pressed(keys.clone()) {
            actions.pressed.push(action.clone());
        }
        if key.any_just_pressed(keys.clone()) {
            actions.just_pressed.push(action.clone());
        }
        if key.any_just_released(keys.clone()) {
            actions.just_released.push(action.clone());
        }
    }
}

#[macro_export]
macro_rules! binds {
    ($($action:expr, $($key:expr),* ;)*) => {
        HashMap::from([
            $(($action, vec![$($key),*]),)*
        ])
    };
}
