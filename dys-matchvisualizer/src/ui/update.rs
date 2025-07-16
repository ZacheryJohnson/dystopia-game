use bevy::app::{App, Plugin};
use bevy::prelude::{IntoScheduleConfigs, Update};
use crate::ui::UiSystems;

pub struct UiUpdate;

impl Plugin for UiUpdate {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            UiUpdate::update
        ).in_set(UiSystems));
    }
}

impl UiUpdate {
    pub fn update() {

    }
}
