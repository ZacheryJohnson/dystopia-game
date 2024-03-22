use std::sync::{Arc, Mutex};

use dys_world::{combatant::combatant::Combatant, team::team::Team, world::World};

use crate::generator::Generator;

pub struct Director {
    world: World
}

impl Director {
    pub fn from_world(world: World) -> Director {
        Director {
            world
        }
    }

    pub fn new() -> Director {
        let generator = Generator::new();

        Director {
            world: generator.generate_world()
        }
    }

    pub fn teams(&self) -> &Vec<Arc<Mutex<Team>>> {
        &self.world.teams
    }

    pub fn combatants(&self) -> &Vec<Arc<Mutex<Combatant>>> {
        &self.world.combatants
    }

    pub fn tick(&self) {
        tracing::info!("Director ticking!");
    }
}