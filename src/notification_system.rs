use std::collections::VecDeque;
use bevy::prelude::*;
use crate::GameActions;
use crate::input_actions::ActionState;

pub struct NotificationSystemPlugin;

impl Plugin for NotificationSystemPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource( Notifications {messages: VecDeque::new(), displayed: None });
    }
}

#[derive(Resource)]
pub struct Notifications {
    messages: VecDeque<Notification>,
    displayed: Option<Notification>,
}

impl Notifications {
    pub fn notify(&mut self, message: Notification) {
        self.messages.push_back(message);
    }
    pub fn next(&mut self) -> bool {
        self.displayed = self.messages.pop_front();
        self.displayed.is_some()
    }
    pub fn has_next(&self) -> bool {
        !self.messages.is_empty() && self.displayed.is_none()
    }
    pub fn clear(&mut self) {
        self.messages.clear();
        self.displayed = None;
    }
}

pub struct Notification {
    from: String,
    message: String,
}

fn notification_system(
    mut commands: Commands,
    mut notifications: ResMut<Notifications>,
    actions: ActionState<GameActions>,
) {
    if notifications.has_next() {
        notifications.next();
    }
    
    if notifications.displayed.is_some() {
        
    }
}