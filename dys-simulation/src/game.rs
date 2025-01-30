use std::sync::{Arc, Mutex};
use dys_world::{arena::{ball_spawn::ArenaBallSpawn, barrier::ArenaBarrier, feature::ArenaFeature, plate::ArenaPlate}, schedule::schedule_game::ScheduleGame};
use rapier3d::prelude::*;

use crate::{
    game_log::GameLog,
    game_objects::game_object::GameObject,
    game_state::GameState,
    game_tick::{GameTick, TickPerformance},
    simulation::simulate_tick,
    simulation::simulation_event::SimulationEvent};

#[derive(Clone)]
pub struct Game {
    pub schedule_game: ScheduleGame,
}

impl Game {
    fn simulate_internal(&self, game_state: GameState) -> GameLog {
        let mut ticks = vec![];

        // Add a "tick 0" for initial state
        // ZJ-TODO: there should probably be a SimulationEvent::InitialState, rather than a bunch of updates
        {
            let (rigid_body_set, _, _) = game_state.physics_sim.sets();
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

            let arena = self.schedule_game.arena.lock().unwrap();
            let arena_features = arena.all_features();
            for feature in arena_features.iter().filter(|feature| feature.shape().is_some()) {
                let shape = feature.shape().unwrap();
                let object_type_id: u32 = {
                    if let Some(barrier) = feature.as_any().downcast_ref::<ArenaBarrier>() {
                        match barrier.pathing_type() {
                            dys_world::arena::feature::NavmeshPathingType::Generate => 1,
                            dys_world::arena::feature::NavmeshPathingType::Skip => 0,
                            dys_world::arena::feature::NavmeshPathingType::Block => 2,
                        }
                    } else if feature.as_any().downcast_ref::<ArenaBallSpawn>().is_some() {
                        3
                    } else if feature.as_any().downcast_ref::<ArenaPlate>().is_some() {
                        4
                    } else {
                        0
                    }
                };
                simulation_events.push(SimulationEvent::ArenaObjectPositionUpdate {
                    object_type_id,  
                    position: *feature.origin(),
                    scale: match shape.shape_type() {
                        ShapeType::Ball => vector![shape.as_ball().unwrap().radius, shape.as_ball().unwrap().radius, shape.as_ball().unwrap().radius],
                        ShapeType::Cuboid => shape.as_cuboid().unwrap().half_extents * 2.0,
                        ShapeType::Capsule => vector![shape.as_capsule().unwrap().radius, shape.as_capsule().unwrap().height(), shape.as_capsule().unwrap().radius],
                        ShapeType::Cylinder => vector![shape.as_cylinder().unwrap().radius, shape.as_cylinder().unwrap().half_height * 2.0, shape.as_cylinder().unwrap().radius],
                        _ => panic!("shape unsupported")
                    },
                    rotation: *feature.rotation(),
                })
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

        let game_state = Arc::new(Mutex::new(game_state));
        loop {
            let new_tick = simulate_tick(game_state.clone());
            let is_end_of_game = new_tick.is_end_of_game();

            ticks.push(new_tick);

            if is_end_of_game {
                break;
            }
        }

        let game_state = game_state.lock().unwrap();
        tracing::info!("Final score: {} - {}", game_state.away_points, game_state.home_points);

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

// Game simulations can be horrendously slow when run in debug mode because of the physics sim
// In the current version of Rust, the below line effectively checks for Release build configurations
#[cfg(not(debug_assertions))]
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