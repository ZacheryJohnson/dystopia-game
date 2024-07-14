use dys_world::schedule::schedule_game::ScheduleGame;

use crate::{game_log::GameLog, game_objects::game_object::GameObject, game_state::GameState, game_tick::{GameTick, TickPerformance}, simulation::simulation_event::SimulationEvent};

#[derive(Clone)]
pub struct Game {
    pub schedule_game: ScheduleGame,
}

impl Game {
    fn simulate_internal(&self, mut game_state: GameState) -> GameLog {
        let mut ticks = vec![];

        // Add a "tick 0" for initial state
        // ZJ-TODO: there should probably be a SimulationEvent::InitialState, rather than a bunch of updates
        {
            let (rigid_body_set, _) = game_state.physics_sim.sets();
            let mut simulation_events = vec![];
            for (combatant_id, combatant_object) in &game_state.combatants {
                let combatant_rb = rigid_body_set.get(combatant_object.rigid_body_handle().unwrap()).unwrap();
                simulation_events.push(SimulationEvent::CombatantPositionUpdate { 
                    combatant_id: *combatant_id,
                    position: *combatant_rb.translation(),
                });
            }
            
            for (ball_id, ball_object) in &game_state.balls {
                let ball_rb = rigid_body_set.get(ball_object.rigid_body_handle().unwrap()).unwrap();
                simulation_events.push(SimulationEvent::BallPositionUpdate { 
                    ball_id: *ball_id,
                    position: *ball_rb.translation(),
                });
            }

            let tick_zero = GameTick {
                tick_number: 0,
                tick_performance: TickPerformance::default(),
                simulation_events,
                is_halftime: false,
                is_end_of_game: false,
            };

            ticks.push(tick_zero);
        }

        loop {
            let new_tick = game_state.tick();
            let is_end_of_game = new_tick.is_end_of_game();

            ticks.push(new_tick);

            if is_end_of_game {
                break;
            }
        }

        GameLog::from_ticks(ticks)
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
        let _ = game.simulate_seeded(seed);
    }
}