use std::sync::{Arc, Mutex};
use dys_world::{arena::{ball_spawn::ArenaBallSpawn, barrier::ArenaBarrier, feature::ArenaFeature, plate::ArenaPlate}, matches::instance::MatchInstance};
use rapier3d::prelude::*;
use dys_world::arena::Arena;
use crate::{
    game_log::GameLog,
    game_objects::game_object::GameObject,
    game_state::GameState,
    game_tick::{GameTick, TickPerformance},
    simulation::simulate_tick,
    simulation::simulation_event::SimulationEvent};

#[derive(Clone)]
pub struct Game {
    pub match_instance: MatchInstance,
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

            let arena = Arena::new_with_testing_defaults();
            // let arena = self.match_instance.arena.lock().unwrap();
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

        GameLog::from_ticks(ticks, game_state)
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
#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};
    use rand::prelude::StdRng;
    use rand::SeedableRng;
    use dys_world::{arena::Arena, schedule::calendar::{Date, Month}, generator::Generator, matches::instance::MatchInstance};

    use crate::game::Game;

    #[test]
    fn test_deterministic_simulations() {
        let world = Generator::new().generate_world(&mut StdRng::from_os_rng());

        let game = Game {
            match_instance: MatchInstance {
                match_id: 0,
                away_team: world.teams[0].clone(),
                home_team: world.teams[1].clone(),
                // arena: Arc::new(Mutex::new(Arena::new_with_testing_defaults())),
                arena_id: 0,
                date: Date(Month::Arguscorp, 1, 10000),
            },
        };
        let seed = &[0; 32];
        let game_1 = game.simulate_seeded(seed);
        let game_2 = game.simulate_seeded(seed);

        assert_eq!(game_1.home_score(), game_2.home_score());
        assert_eq!(game_1.away_score(), game_2.away_score());
    }
}