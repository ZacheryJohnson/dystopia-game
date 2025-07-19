use bevy::prelude::Resource;
use dys_simulation::game_log::GameLog;
use dys_simulation::game_tick::GameTickNumber;
use dys_world::world::World;
use web_time::Instant;

#[derive(Clone, Debug)]
pub enum VisualizationMode {
    /// The visualization will not progress
    Paused,

    /// The visualization will progress only when the viewer instructs it to
    Step,

    /// The visualization will progress without user intervention
    Play,
}

#[derive(Debug, Resource)]
pub struct VisualizationState {
    pub  should_exit: bool,
    pub game_log: Option<GameLog>,
    pub world_state: Option<World>,
    pub current_tick: GameTickNumber,
    pub last_update_time: Instant,
    pub home_score: u16,
    pub home_score_diff_from_last_tick: i16,
    pub away_score: u16,
    pub away_score_diff_from_last_tick: i16,
    pub end_of_game: bool,
    pub mode: VisualizationMode,
}

impl VisualizationState {
    pub fn from(game_log: GameLog, world_state: World) -> Self {
        let mut vis_state = VisualizationState::default();
        vis_state.game_log = Some(game_log);
        vis_state.world_state = Some(world_state);

        #[cfg(not(target_family="wasm"))]
        let visualization_mode = VisualizationMode::Step;

        #[cfg(target_family="wasm")]
        let visualization_mode = VisualizationMode::Play;

        vis_state.mode = visualization_mode;

        vis_state
    }
}

impl Default for VisualizationState {
    fn default() -> Self {
        VisualizationState {
            should_exit: false,
            game_log: None,
            world_state: None,
            current_tick: 0,
            last_update_time: Instant::now(),
            home_score: 0,
            home_score_diff_from_last_tick: 0,
            away_score: 0,
            away_score_diff_from_last_tick: 0,
            end_of_game: false,
            mode: VisualizationMode::Paused,
        }
    }
}