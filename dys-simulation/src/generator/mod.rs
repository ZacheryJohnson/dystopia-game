use std::sync::{Arc, Mutex};
use rand::distributions::Distribution;
use dys_world::combatant::instance::CombatantInstance;
use dys_world::combatant::limb::{Limb, LimbModifier, LimbType};
use dys_world::attribute::instance::AttributeInstance;
use dys_world::attribute::attribute_type::AttributeType;
use dys_world::team::instance::TeamInstance;
use dys_world::world::World;
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use rand_distr::Normal;

pub struct Generator {
    given_names: Vec<String>,
    surnames: Vec<String>,
    team_names: Vec<String>,
}

impl Default for Generator {
    fn default() -> Self {
        Self::new()
    }
}

impl Generator {
    pub fn new() -> Generator {
        let given_names: Vec<String> = include_str!("../../data/given_names.txt")
            .split_whitespace()
            .map(|s| s.to_owned())
            .collect();

        let surnames: Vec<String> = include_str!("../../data/surnames.txt")
            .split_whitespace()
            .map(|s| s.to_owned())
            .collect();

        let team_names = vec![
            "Alpha".to_string(),
            "Beta".to_string(),
            "Gamma".to_string(),
            "Delta".to_string(),
            "Epsilon".to_string(),
            "Zeta".to_string(),
            "Eta".to_string(),
            "Theta".to_string(),
            "Iota".to_string(),
            "Kappa".to_string(),
            "Lambda".to_string(),
            "Mu".to_string(),
        ];

        Generator {
            given_names,
            surnames,
            team_names
        }
    }

    fn generate_limbs(&self, rng: &mut ThreadRng) -> Vec<Limb> {
        let mut generate_value_around_fn = |mean| {
            let normal_distribution = Normal::new(mean, 3.0).unwrap();
            normal_distribution.sample(rng)
        };

        vec![
            Limb {
                limb_type: LimbType::Torso, 
                modifiers: vec![ LimbModifier::default_with_attributes(&[
                    AttributeInstance::new(AttributeType::Strength, generate_value_around_fn(10.0)),
                    AttributeInstance::new(AttributeType::Constitution, generate_value_around_fn(20.0)),
                ])],
                child_limbs: vec![
                    Limb { 
                        limb_type: LimbType::Head, 
                        modifiers: vec![ LimbModifier::default_with_attributes(&[
                            AttributeInstance::new(AttributeType::Cognition, generate_value_around_fn(30.0))
                        ])],
                        child_limbs: vec![
                            Limb {
                                limb_type: LimbType::Eye,
                                modifiers: vec![ LimbModifier::default_with_attributes(&[
                                    AttributeInstance::new(AttributeType::Cognition, generate_value_around_fn(10.0))
                                ])],
                                child_limbs: vec![]
                            },
                            Limb {
                                limb_type: LimbType::Eye,
                                modifiers: vec![ LimbModifier::default_with_attributes(&[
                                    AttributeInstance::new(AttributeType::Cognition, generate_value_around_fn(10.0))
                                ])],
                                child_limbs: vec![]
                            },
                            Limb {
                                limb_type: LimbType::Nose,
                                modifiers: vec![ LimbModifier::default_with_attributes(&[
                                    AttributeInstance::new(AttributeType::Presence, generate_value_around_fn(10.0))
                                ])],
                                child_limbs: vec![]
                            },
                            Limb {
                                limb_type: LimbType::Mouth,
                                modifiers: vec![ LimbModifier::default_with_attributes(&[
                                    AttributeInstance::new(AttributeType::Communication, generate_value_around_fn(10.0))
                                ])],
                                child_limbs: vec![]
                            },
                        ]
                    },
                    Limb {
                        limb_type: LimbType::Arm,
                        modifiers: vec![ LimbModifier::default_with_attributes(&[
                            AttributeInstance::new(AttributeType::Coordination, generate_value_around_fn(5.0)),
                            AttributeInstance::new(AttributeType::Dexterity, generate_value_around_fn(10.0)),
                            AttributeInstance::new(AttributeType::Stability, generate_value_around_fn(5.0)),
                        ])],
                        child_limbs: vec![
                            Limb {
                                limb_type: LimbType::Hand,
                                modifiers: vec![ LimbModifier::default_with_attributes(&[
                                    AttributeInstance::new(AttributeType::Communication, generate_value_around_fn(5.0))
                                ])],
                                child_limbs: vec![
                                    Limb {
                                        limb_type: LimbType::Finger,
                                        modifiers: vec![ LimbModifier::default_with_attributes(&[
                                            AttributeInstance::new(AttributeType::Communication, generate_value_around_fn(1.0))
                                        ])],
                                        child_limbs: vec![]
                                    },
                                    Limb {
                                        limb_type: LimbType::Finger,
                                        modifiers: vec![ LimbModifier::default_with_attributes(&[
                                            AttributeInstance::new(AttributeType::Communication, generate_value_around_fn(1.0))
                                        ])],
                                        child_limbs: vec![]
                                    },
                                    Limb {
                                        limb_type: LimbType::Finger,
                                        modifiers: vec![ LimbModifier::default_with_attributes(&[
                                            AttributeInstance::new(AttributeType::Communication, generate_value_around_fn(1.0))
                                        ])],
                                        child_limbs: vec![]
                                    },
                                    Limb {
                                        limb_type: LimbType::Finger,
                                        modifiers: vec![ LimbModifier::default_with_attributes(&[
                                            AttributeInstance::new(AttributeType::Communication, generate_value_around_fn(1.0))
                                        ])],
                                        child_limbs: vec![]
                                    },
                                    Limb {
                                        limb_type: LimbType::Finger,
                                        modifiers: vec![ LimbModifier::default_with_attributes(&[
                                            AttributeInstance::new(AttributeType::Communication, generate_value_around_fn(1.0))
                                        ])],
                                        child_limbs: vec![]
                                    },
                                ]
                            }
                        ],
                    },
                    Limb {
                        limb_type: LimbType::Arm,
                        modifiers: vec![ LimbModifier::default_with_attributes(&[
                            AttributeInstance::new(AttributeType::Coordination, generate_value_around_fn(5.0)),
                            AttributeInstance::new(AttributeType::Dexterity, generate_value_around_fn(10.0)),
                            AttributeInstance::new(AttributeType::Stability, generate_value_around_fn(5.0)),
                        ])],
                        child_limbs: vec![
                            Limb {
                                limb_type: LimbType::Hand,
                                modifiers: vec![ LimbModifier::default_with_attributes(&[
                                    AttributeInstance::new(AttributeType::Communication, generate_value_around_fn(5.0))
                                ])],
                                child_limbs: vec![
                                    Limb {
                                        limb_type: LimbType::Finger,
                                        modifiers: vec![ LimbModifier::default_with_attributes(&[
                                            AttributeInstance::new(AttributeType::Communication, generate_value_around_fn(1.0))
                                        ])],
                                        child_limbs: vec![]
                                    },
                                    Limb {
                                        limb_type: LimbType::Finger,
                                        modifiers: vec![ LimbModifier::default_with_attributes(&[
                                            AttributeInstance::new(AttributeType::Communication, generate_value_around_fn(1.0))
                                        ])],
                                        child_limbs: vec![]
                                    },
                                    Limb {
                                        limb_type: LimbType::Finger,
                                        modifiers: vec![ LimbModifier::default_with_attributes(&[
                                            AttributeInstance::new(AttributeType::Communication, generate_value_around_fn(1.0))
                                        ])],
                                        child_limbs: vec![]
                                    },
                                    Limb {
                                        limb_type: LimbType::Finger,
                                        modifiers: vec![ LimbModifier::default_with_attributes(&[
                                            AttributeInstance::new(AttributeType::Communication, generate_value_around_fn(1.0))
                                        ])],
                                        child_limbs: vec![]
                                    },
                                    Limb {
                                        limb_type: LimbType::Finger,
                                        modifiers: vec![ LimbModifier::default_with_attributes(&[
                                            AttributeInstance::new(AttributeType::Communication, generate_value_around_fn(1.0))
                                        ])],
                                        child_limbs: vec![]
                                    },
                                ]
                            }
                        ],
                    },
                    Limb {
                        limb_type: LimbType::Leg,
                        modifiers: vec![ LimbModifier::default_with_attributes(&[
                            AttributeInstance::new(AttributeType::Coordination, generate_value_around_fn(10.0)),
                            AttributeInstance::new(AttributeType::Dexterity, generate_value_around_fn(2.0)),
                            AttributeInstance::new(AttributeType::Stability, generate_value_around_fn(5.0)),
                            AttributeInstance::new(AttributeType::Strength, generate_value_around_fn(2.0)),
                        ])],
                        child_limbs: vec![
                            Limb {
                                limb_type: LimbType::Knee,
                                modifiers: vec![ LimbModifier::default_with_attributes(&[
                                    AttributeInstance::new(AttributeType::Dexterity, generate_value_around_fn(8.0))
                                ])],
                                child_limbs: vec![]
                            },
                            Limb {
                                limb_type: LimbType::Foot,
                                modifiers: vec![ LimbModifier::default_with_attributes(&[
                                    AttributeInstance::new(AttributeType::Dexterity, generate_value_around_fn(5.0)),
                                    AttributeInstance::new(AttributeType::Stability, generate_value_around_fn(5.0)),
                                ])],
                                child_limbs: vec![
                                    Limb {
                                        limb_type: LimbType::Toe,
                                        modifiers: vec![ LimbModifier::default_with_attributes(&[
                                            AttributeInstance::new(AttributeType::Stability, generate_value_around_fn(1.0))
                                        ])],
                                        child_limbs: vec![]
                                    },
                                    Limb {
                                        limb_type: LimbType::Toe,
                                        modifiers: vec![ LimbModifier::default_with_attributes(&[
                                            AttributeInstance::new(AttributeType::Stability, generate_value_around_fn(1.0))
                                        ])],
                                        child_limbs: vec![]
                                    },
                                    Limb {
                                        limb_type: LimbType::Toe,
                                        modifiers: vec![ LimbModifier::default_with_attributes(&[
                                            AttributeInstance::new(AttributeType::Stability, generate_value_around_fn(1.0))
                                        ])],
                                        child_limbs: vec![]
                                    },
                                    Limb {
                                        limb_type: LimbType::Toe,
                                        modifiers: vec![ LimbModifier::default_with_attributes(&[
                                            AttributeInstance::new(AttributeType::Stability, generate_value_around_fn(1.0))
                                        ])],
                                        child_limbs: vec![]
                                    },
                                    Limb {
                                        limb_type: LimbType::Toe,
                                        modifiers: vec![ LimbModifier::default_with_attributes(&[
                                            AttributeInstance::new(AttributeType::Stability, generate_value_around_fn(1.0))
                                        ])],
                                        child_limbs: vec![]
                                    },
                                ]
                            }
                        ],
                    },
                    Limb {
                        limb_type: LimbType::Leg,
                        modifiers: vec![ LimbModifier::default_with_attributes(&[
                            AttributeInstance::new(AttributeType::Coordination, generate_value_around_fn(10.0)),
                            AttributeInstance::new(AttributeType::Dexterity, generate_value_around_fn(2.0)),
                            AttributeInstance::new(AttributeType::Stability, generate_value_around_fn(5.0)),
                            AttributeInstance::new(AttributeType::Strength, generate_value_around_fn(2.0)),
                        ])],
                        child_limbs: vec![
                            Limb {
                                limb_type: LimbType::Knee,
                                modifiers: vec![ LimbModifier::default_with_attributes(&[
                                    AttributeInstance::new(AttributeType::Dexterity, generate_value_around_fn(8.0))
                                ])],
                                child_limbs: vec![]
                            },
                            Limb {
                                limb_type: LimbType::Foot,
                                modifiers: vec![ LimbModifier::default_with_attributes(&[
                                    AttributeInstance::new(AttributeType::Dexterity, generate_value_around_fn(5.0)),
                                    AttributeInstance::new(AttributeType::Stability, generate_value_around_fn(5.0)),
                                ])],
                                child_limbs: vec![
                                    Limb {
                                        limb_type: LimbType::Toe,
                                        modifiers: vec![ LimbModifier::default_with_attributes(&[
                                            AttributeInstance::new(AttributeType::Stability, generate_value_around_fn(1.0))
                                        ])],
                                        child_limbs: vec![]
                                    },
                                    Limb {
                                        limb_type: LimbType::Toe,
                                        modifiers: vec![ LimbModifier::default_with_attributes(&[
                                            AttributeInstance::new(AttributeType::Stability, generate_value_around_fn(1.0))
                                        ])],
                                        child_limbs: vec![]
                                    },
                                    Limb {
                                        limb_type: LimbType::Toe,
                                        modifiers: vec![ LimbModifier::default_with_attributes(&[
                                            AttributeInstance::new(AttributeType::Stability, generate_value_around_fn(1.0))
                                        ])],
                                        child_limbs: vec![]
                                    },
                                    Limb {
                                        limb_type: LimbType::Toe,
                                        modifiers: vec![ LimbModifier::default_with_attributes(&[
                                            AttributeInstance::new(AttributeType::Stability, generate_value_around_fn(1.0))
                                        ])],
                                        child_limbs: vec![]
                                    },
                                    Limb {
                                        limb_type: LimbType::Toe,
                                        modifiers: vec![ LimbModifier::default_with_attributes(&[
                                            AttributeInstance::new(AttributeType::Stability, generate_value_around_fn(1.0))
                                        ])],
                                        child_limbs: vec![]
                                    },
                                ]
                            }
                        ],
                    },
                ]
            }, 
        ]
    } 

    fn generate_combatant(&self, id: u64, rng: &mut ThreadRng) -> CombatantInstance {
        let combatant_given_name = self.given_names.choose(rng).unwrap().to_owned();
        let combatant_surname = self.surnames.choose(rng).unwrap().to_owned();

        // ZJ-TODO: add small chance for hyphenated surnames

        let combatant_name = format!("{combatant_given_name} {combatant_surname}");
    
        let combatant_limbs = self.generate_limbs(rng);
    
        CombatantInstance {
            id,
            name: combatant_name,
            limbs: combatant_limbs,
        }
    }
    
    pub fn generate_combatants(&self, count: u64) -> Vec<Arc<Mutex<CombatantInstance>>> {
        let mut combatants = vec![];
    
        let mut thread_rng = rand::thread_rng();

        for i in 0..count {
            let new_combatant = self.generate_combatant(i, &mut thread_rng);
            combatants.push(Arc::new(Mutex::new(new_combatant)));
        }
    
        combatants
    }

    pub fn generate_world(&self) -> World {
        // ZJ-TODO: this should be config driven
        let number_of_teams = 12;
        let players_per_team = 8;
        let total_combatants_to_generate = number_of_teams * players_per_team;

        let combatants = self.generate_combatants(total_combatants_to_generate);
        let teams = combatants
            .clone()
            .chunks(players_per_team as usize)
            .enumerate()
            .map(|(id, combatants)| TeamInstance {
                id: id as u64,
                name: self.team_names[id].clone(),
                combatants: combatants.to_vec(),
            })
            .map(|team| Arc::new(Mutex::new(team)))
            .collect();
            
        World {
            combatants,
            teams
        }
    }
}
