use bevy::app::App;
use bevy::prelude::{Plugin, SystemSet};

mod setup;
mod update;

#[derive(Clone, Debug, PartialEq, Eq, Hash, SystemSet)]
pub struct VisualizerSystems;

pub struct VisualizerPlugin;
impl Plugin for VisualizerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            setup::VisualizerSetup,
            update::VisualizerUpdate,
        ));
    }
}

pub mod components;
pub mod resources;