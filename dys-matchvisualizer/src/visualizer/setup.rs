use bevy::app::App;
use bevy::prelude::Plugin;
use crate::visualizer::resources::VisualizationState;

pub struct VisualizerSetup;

impl Plugin for VisualizerSetup {
    fn build(&self, app: &mut App) {
        app.init_resource::<VisualizationState>();
    }
}