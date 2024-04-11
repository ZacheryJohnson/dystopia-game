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

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use dys_world::{arena::Arena, schedule::{calendar::{Date, Month}, schedule_game::ScheduleGame}};

    use crate::{game::Game, generator::Generator};

    #[test]
    fn test_speed() {
        let world = Generator::new().generate_world();

        let game = Game {
            schedule_game: ScheduleGame {
                away_team: world.teams[0].clone(),
                home_team: world.teams[1].clone(),
                arena: Arc::new(Mutex::new(Arena::new_with_testing_defaults())),
                date: Date(Month::Arguscorp, 1, 10000),
            },
        };
        let seed = &[0; 32];
        let game_log = game.simulate_seeded(seed);
        let physics_duration_micros: u128 = game_log.ticks.iter().map(|tick| tick.physics_duration.as_micros()).sum();
        let balls_duration_micros: u128 = game_log.ticks.iter().map(|tick| tick.balls_duration.as_micros()).sum();
        let total_duration_micros: u128 = game_log.ticks.iter().map(|tick| tick.tick_duration.as_micros()).sum();

        for tick in &game_log.ticks {
            for simulation_event in &tick.simulation_events {
                println!("Tick {}: {:?}", tick.tick_number, simulation_event);
            }
        }

        println!("Simulation duration: {total_duration_micros} microseconds ({physics_duration_micros} in physics, {balls_duration_micros} in balls)");
    }
}