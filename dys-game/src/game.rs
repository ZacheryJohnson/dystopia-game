use dys_world::schedule::schedule_game::ScheduleGame;

use crate::{game_log::GameLog, game_state::GameState};

#[derive(Clone)]
pub struct Game {
    pub schedule_game: ScheduleGame,
}

impl Game {
    fn simulate_internal(&self, mut game_state: GameState) -> GameLog {
        let mut ticks = vec![];
        loop {
            let new_tick = game_state.tick();
            let is_end_of_game = new_tick.is_end_of_game();

            ticks.push(new_tick);

            if is_end_of_game {
                break;
            }
        }

        GameLog { 
            ticks, 
            schedule_game: self.schedule_game.clone()
        }
    }

    pub fn simulate(&self) -> GameLog {
        let game_state = GameState::from_game(self.clone());
        self.simulate_internal(game_state)
    }

    pub fn simulate_seeded(&self, seed: &[u8; 32]) -> GameLog {
        let game_state = GameState::from_game_seeded(self.clone(), seed);
        self.simulate_internal(game_state)
    }
}