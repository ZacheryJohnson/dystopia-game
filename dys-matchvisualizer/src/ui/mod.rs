use bevy::app::App;
use bevy::prelude::{Plugin, SystemSet};

mod setup;
mod update;

pub struct UiPlugin;

#[derive(Clone, Debug, PartialEq, Eq, Hash, SystemSet)]
pub struct UiSystems;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            setup::UiSetup,
            update::UiUpdate,
        ));
    }
}

pub mod components;