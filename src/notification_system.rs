use bevy::prelude::*;
use std::collections::VecDeque;

pub struct NotificationSystemPlugin;

impl Plugin for NotificationSystemPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Notifications {
            messages: VecDeque::new(),
            displayed: None,
        });
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
    mut notifications: ResMut<Notifications>,
) {
    if notifications.has_next() {
        notifications.next();
    }

    if notifications.displayed.is_some() {}
}
