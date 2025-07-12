use bevy::app::App;
use bevy::prelude::Plugin;

mod setup;
mod update;

pub struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            setup::UiSetup,
            update::UiUpdate,
        ));
    }
}

pub mod components;