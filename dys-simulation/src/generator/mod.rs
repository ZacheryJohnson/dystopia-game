use std::sync::{Arc, Mutex};

use dys_world::combatant::definition::CombatantDefinition;
use dys_world::combatant::limb::{Limb, LimbModifier, LimbModifierType, LimbType, ModifierAcquisitionMethod};
use dys_world::stat::definition::{StatDefinition, StatType};
use dys_world::team::definition::TeamDefinition;
use dys_world::world::World;
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;

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

    fn default_limbs(&self) -> Vec<Limb> {
        vec![
            Limb {
                limb_type: LimbType::Torso, 
                modifiers: vec![ LimbModifier { modifier_type: LimbModifierType::Regular, acquisition: ModifierAcquisitionMethod::Inherent, stats: vec![StatDefinition { stat_type: StatType::Strength, value: 10.0 }] }],
                child_limbs: vec![
                    Limb { 
                        limb_type: LimbType::Head, 
                        modifiers: vec![ LimbModifier { modifier_type: LimbModifierType::Regular, acquisition: ModifierAcquisitionMethod::Inherent, stats: vec![StatDefinition { stat_type: StatType::Cognition, value: 30.0 }] }],
                        child_limbs: vec![
                            Limb {
                                limb_type: LimbType::Eye,
                                modifiers: vec![ LimbModifier { modifier_type: LimbModifierType::Regular, acquisition: ModifierAcquisitionMethod::Inherent, stats: vec![StatDefinition { stat_type: StatType::Reaction, value: 10.0 }] }],
                                child_limbs: vec![]
                            },
                            Limb {
                                limb_type: LimbType::Eye,
                                modifiers: vec![ LimbModifier { modifier_type: LimbModifierType::Regular, acquisition: ModifierAcquisitionMethod::Inherent, stats: vec![StatDefinition { stat_type: StatType::Reaction, value: 10.0 }] }],
                                child_limbs: vec![]
                            },
                            Limb {
                                limb_type: LimbType::Nose,
                                modifiers: vec![ /* no modifiers? what useful things do noses provide in sports? */ ],
                                child_limbs: vec![]
                            },
                            Limb {
                                limb_type: LimbType::Mouth,
                                modifiers: vec![ /* teamwork? communication? */ ],
                                child_limbs: vec![]
                            },
                        ]
                    },
                    Limb {
                        limb_type: LimbType::Arm,
                        modifiers: vec![/* TODO */],
                        child_limbs: vec![
                            Limb {
                                limb_type: LimbType::Hand,
                                modifiers: vec![/* TODO */],
                                child_limbs: vec![
                                    Limb {
                                        limb_type: LimbType::Finger,
                                        modifiers: vec![/* TODO */],
                                        child_limbs: vec![]
                                    },
                                    Limb {
                                        limb_type: LimbType::Finger,
                                        modifiers: vec![/* TODO */],
                                        child_limbs: vec![]
                                    },
                                    Limb {
                                        limb_type: LimbType::Finger,
                                        modifiers: vec![/* TODO */],
                                        child_limbs: vec![]
                                    },
                                    Limb {
                                        limb_type: LimbType::Finger,
                                        modifiers: vec![/* TODO */],
                                        child_limbs: vec![]
                                    },
                                    Limb {
                                        limb_type: LimbType::Finger,
                                        modifiers: vec![/* TODO */],
                                        child_limbs: vec![]
                                    },
                                ]
                            }
                        ],
                    },
                    Limb {
                        limb_type: LimbType::Arm,
                        modifiers: vec![/* TODO */],
                        child_limbs: vec![
                            Limb {
                                limb_type: LimbType::Hand,
                                modifiers: vec![/* TODO */],
                                child_limbs: vec![
                                    Limb {
                                        limb_type: LimbType::Finger,
                                        modifiers: vec![/* TODO */],
                                        child_limbs: vec![]
                                    },
                                    Limb {
                                        limb_type: LimbType::Finger,
                                        modifiers: vec![/* TODO */],
                                        child_limbs: vec![]
                                    },
                                    Limb {
                                        limb_type: LimbType::Finger,
                                        modifiers: vec![/* TODO */],
                                        child_limbs: vec![]
                                    },
                                    Limb {
                                        limb_type: LimbType::Finger,
                                        modifiers: vec![/* TODO */],
                                        child_limbs: vec![]
                                    },
                                    Limb {
                                        limb_type: LimbType::Finger,
                                        modifiers: vec![/* TODO */],
                                        child_limbs: vec![]
                                    },
                                ]
                            }
                        ],
                    },
                    Limb {
                        limb_type: LimbType::Leg,
                        modifiers: vec![/* TODO */],
                        child_limbs: vec![
                            Limb {
                                limb_type: LimbType::Knee,
                                modifiers: vec![/* TODO */],
                                child_limbs: vec![]
                            },
                            Limb {
                                limb_type: LimbType::Foot,
                                modifiers: vec![/* TODO */],
                                child_limbs: vec![
                                    Limb {
                                        limb_type: LimbType::Toe,
                                        modifiers: vec![/* TODO */],
                                        child_limbs: vec![]
                                    },
                                    Limb {
                                        limb_type: LimbType::Toe,
                                        modifiers: vec![/* TODO */],
                                        child_limbs: vec![]
                                    },
                                    Limb {
                                        limb_type: LimbType::Toe,
                                        modifiers: vec![/* TODO */],
                                        child_limbs: vec![]
                                    },
                                    Limb {
                                        limb_type: LimbType::Toe,
                                        modifiers: vec![/* TODO */],
                                        child_limbs: vec![]
                                    },
                                    Limb {
                                        limb_type: LimbType::Toe,
                                        modifiers: vec![/* TODO */],
                                        child_limbs: vec![]
                                    },
                                ]
                            }
                        ],
                    },
                    Limb {
                        limb_type: LimbType::Leg,
                        modifiers: vec![/* TODO */],
                        child_limbs: vec![
                            Limb {
                                limb_type: LimbType::Knee,
                                modifiers: vec![/* TODO */],
                                child_limbs: vec![]
                            },
                            Limb {
                                limb_type: LimbType::Foot,
                                modifiers: vec![/* TODO */],
                                child_limbs: vec![
                                    Limb {
                                        limb_type: LimbType::Toe,
                                        modifiers: vec![/* TODO */],
                                        child_limbs: vec![]
                                    },
                                    Limb {
                                        limb_type: LimbType::Toe,
                                        modifiers: vec![/* TODO */],
                                        child_limbs: vec![]
                                    },
                                    Limb {
                                        limb_type: LimbType::Toe,
                                        modifiers: vec![/* TODO */],
                                        child_limbs: vec![]
                                    },
                                    Limb {
                                        limb_type: LimbType::Toe,
                                        modifiers: vec![/* TODO */],
                                        child_limbs: vec![]
                                    },
                                    Limb {
                                        limb_type: LimbType::Toe,
                                        modifiers: vec![/* TODO */],
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

    fn generate_combatant(&self, id: u64, rng: &mut ThreadRng) -> CombatantDefinition {
        let combatant_given_name = self.given_names.choose(rng).unwrap().to_owned();
        let combatant_surname = self.surnames.choose(rng).unwrap().to_owned();

        // ZJ-TODO: add small chance for hyphenated surnames

        let combatant_name = format!("{combatant_given_name} {combatant_surname}");

        // ZJ-TODO: add slight variance in limbs
    
        let combatant_limbs = self.default_limbs();
    
        CombatantDefinition {
            id,
            name: combatant_name,
            limbs: combatant_limbs,
        }
    }
    
    pub fn generate_combatants(&self, count: u64) -> Vec<Arc<Mutex<CombatantDefinition>>> {
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
            .map(|(id, combatants)| TeamDefinition {
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
