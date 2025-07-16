use bevy::app::App;
use bevy::prelude::{IntoScheduleConfigs, Plugin, Query, Res, Update, Visibility, With};
use crate::visualizer::resources::VisualizationState;
use crate::visualizer::VisualizerSystems;

pub struct VisualizerUpdate;

impl Plugin for VisualizerUpdate {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
           update_postgame_scoreboard,
        ).in_set(VisualizerSystems));
    }
}

fn update_postgame_scoreboard(
    mut query: Query<&mut Visibility, With<crate::ui::components::PostgameScoreboard>>,
    vis_state: Res<VisualizationState>,
) {
    if vis_state.should_exit {
        return;
    }

    let Ok(mut visibility) = query.single_mut() else {
        return;
    };

    if vis_state.end_of_game {
        *visibility = Visibility::Visible;
    } else {
        *visibility = Visibility::Hidden;
    }
}