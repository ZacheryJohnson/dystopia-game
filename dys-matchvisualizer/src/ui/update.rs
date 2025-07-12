use bevy::app::{App, Plugin};
use bevy::prelude::Update;

pub struct UiUpdate;

impl Plugin for UiUpdate {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, UiUpdate::update);
    }
}

impl UiUpdate {
    pub fn update() {

    }
}
