use std::ops::RangeInclusive;
use std::sync::{Arc, Mutex};
use rand::prelude::IteratorRandom;
use rand::Rng;
use rand_distr::Distribution;
use crate::combatant::instance::CombatantInstance;
use crate::combatant::limb::{Limb, LimbModifier, LimbType};
use crate::attribute::instance::AttributeInstance;
use crate::attribute::attribute_type::AttributeType;
use crate::team::instance::TeamInstance;
use crate::world::World;
use rand::seq::SliceRandom;
use rand_distr::Normal;
use crate::arena::Arena;
use crate::matches::instance::MatchInstance;
use crate::proposal::{Proposal, ProposalEffect, ProposalOption};
use crate::schedule::calendar::{Date, Month};
use crate::schedule::season::Season;
use crate::schedule::series::{Series, SeriesType};

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

    fn generate_limbs(&self, rng: &mut impl Rng) -> Vec<Limb> {
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

    fn generate_combatant(&self, id: u64, rng: &mut impl Rng) -> CombatantInstance {
        let combatant_given_name = self.given_names.iter().choose(rng).unwrap().to_owned();
        let combatant_surname = self.surnames.iter().choose(rng).unwrap().to_owned();

        // ZJ-TODO: add small chance for hyphenated surnames

        let combatant_name = format!("{combatant_given_name} {combatant_surname}");
    
        let combatant_limbs = self.generate_limbs(rng);
    
        CombatantInstance {
            id,
            name: combatant_name,
            limbs: combatant_limbs,
            effect_modifiers: vec![]
        }
    }
    
    pub fn generate_combatants(&self, count: u64, rng: &mut impl Rng) -> Vec<Arc<Mutex<CombatantInstance>>> {
        let mut combatants = vec![];

        for i in 0..count {
            let new_combatant = self.generate_combatant(i, rng);
            combatants.push(Arc::new(Mutex::new(new_combatant)));
        }
    
        combatants
    }

    pub fn generate_world(&self, rng: &mut impl Rng) -> World {
        // ZJ-TODO: this should be config driven
        let number_of_teams = 4;
        let players_per_team = 5;
        let total_combatants_to_generate = number_of_teams * players_per_team;

        let combatants = self.generate_combatants(total_combatants_to_generate, rng);
        let teams = combatants
            .clone()
            .chunks(players_per_team as usize)
            .enumerate()
            .map(|(id, combatants)| TeamInstance {
                id: id as u64 + 1,
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

    pub fn generate_season(
        &self,
        rng: &mut impl Rng,
        world: &World
    ) -> Season {
        // ZJ-TODO: I'd love for this to be more interesting
        // For now, just do a simple round-robin of 3 game series

        const SERIES_COUNT: u16 = 33;
        const SERIES_LEN_RANGE: RangeInclusive<u16> = 3..=3;
        let series_len = SERIES_LEN_RANGE.choose(rng).unwrap();

        // Create a first-pass schedule: we'll make changes + validate after

        let mut all_series = vec![];

        let fixed_team = world.teams.first().unwrap().to_owned();
        let mut rotating_teams = Vec::from_iter(world
            .teams
            .iter()
            .skip(1)
            .map(|arc| arc.to_owned())
        );

        let mut match_id = 0;
        for series_idx in 0..SERIES_COUNT {
            let swap_fixed_matchup = series_idx % 2 >= 1;
            let swap_alt_matchup = series_idx % 4 >= 2;

            let fixed_opponent = rotating_teams.pop().unwrap().to_owned();

            let mut fixed_series_matches = vec![];
            for game_idx in 0..series_len {
                match_id += 1;
                fixed_series_matches.push(Arc::new(Mutex::new(MatchInstance {
                    match_id,
                    away_team: if swap_fixed_matchup { fixed_team.clone() } else { fixed_opponent.clone() },
                    home_team: if swap_fixed_matchup { fixed_opponent.clone() } else { fixed_team.clone() },
                    // arena: Arc::new(Mutex::new(Arena::new_with_testing_defaults())), // ZJ-TODO
                    arena_id: 0,
                    date: Date(
                        Month::Arguscorp,
                        1 + (series_len * series_idx) as u32 + game_idx as u32,
                        10000
                    ),
                })));
            }

            let fixed_series = Series {
                matches: fixed_series_matches,
                series_type: SeriesType::Normal,
            };

            let alt_opponent_1 = rotating_teams.pop().unwrap().to_owned();
            let alt_opponent_2 = rotating_teams.pop().unwrap().to_owned();

            let mut alt_series_matches = vec![];
            for game_idx in 0..series_len {
                match_id += 1;
                alt_series_matches.push(Arc::new(Mutex::new(MatchInstance {
                    match_id,
                    away_team: if swap_alt_matchup { alt_opponent_1.clone() } else { alt_opponent_2.clone() },
                    home_team: if swap_alt_matchup { alt_opponent_2.clone() } else { alt_opponent_1.clone() },
                    // arena: Arc::new(Mutex::new(Arena::new_with_testing_defaults())), // ZJ-TODO
                    arena_id: 0,
                    date: Date(
                        Month::Arguscorp,
                        1 + (series_len * series_idx) as u32 + game_idx as u32,
                        10000
                    ),
                })));
            }

            let alt_series = Series {
                matches: alt_series_matches,
                series_type: SeriesType::Normal,
            };

            all_series.push(fixed_series);
            all_series.push(alt_series);

            // When rotating teams:
            // - fixed opponent -> alt 1
            // - alt 1 -> alt 2
            // - alt 2 -> fixed opponent
            // Because we pop from the back, we want vec![new alt 2, new alt 1, new fixed opponent]
            // So we'll push in inverse order: Vec <- current alt 1 <- current fixed opponent <- current alt 2
            rotating_teams.push(alt_opponent_1);
            rotating_teams.push(fixed_opponent);
            rotating_teams.push(alt_opponent_2);
        }

        Season::new(all_series)
    }

    pub fn generate_proposals(
        &self,
        rng: &mut impl Rng,
        world: &World
    ) -> Vec<Proposal> {
        let mut proposals = vec![];

        let mut proposal_id = 0;
        for team in &world.teams {
            proposal_id += 1;

            let team_instance = team.lock().unwrap();
            let team_name = team_instance.name.to_owned();

            let combatants = team_instance.combatants.clone().into_iter()
                .take(3)
                .collect::<Vec<Arc<Mutex<CombatantInstance>>>>();
            let combatant_1_name = combatants[0].lock().unwrap().name.to_owned();
            let combatant_2_name = combatants[1].lock().unwrap().name.to_owned();
            let combatant_3_name = combatants[2].lock().unwrap().name.to_owned();

            proposals.push(
                Proposal {
                    id: proposal_id,
                    name: format!("Supercharge {team_name} Player"),
                    description: "Pick a combatant to supercharge for a match.".to_string(),
                    options: vec![
                        ProposalOption {
                            id: 1,
                            name: combatant_1_name,
                            description: "".to_string(),
                            effects: vec![
                                ProposalEffect::CombatantTemporaryAttributeBonus {
                                    combatant_instance_id: combatants[0].lock().unwrap().id,
                                    attribute_instance_bonus: AttributeInstance::new(AttributeType::Dexterity, 100.0),
                                }
                            ],
                        },
                        ProposalOption {
                            id: 2,
                            name: combatant_2_name,
                            description: "".to_string(),
                            effects: vec![
                                ProposalEffect::CombatantTemporaryAttributeBonus {
                                    combatant_instance_id: combatants[1].lock().unwrap().id,
                                    attribute_instance_bonus: AttributeInstance::new(AttributeType::Dexterity, 100.0),
                                }
                            ],
                        },
                        ProposalOption {
                            id: 3,
                            name: combatant_3_name,
                            description: "".to_string(),
                            effects: vec![
                                ProposalEffect::CombatantTemporaryAttributeBonus {
                                    combatant_instance_id: combatants[2].lock().unwrap().id,
                                    attribute_instance_bonus: AttributeInstance::new(AttributeType::Dexterity, 100.0),
                                }
                            ],
                        },
                    ],
                }
            );
        }

        proposals
    }
}

#[cfg(test)]
mod tests {
    use rand::thread_rng;
    use super::*;

    #[test]
    fn generate_season() {
        let generator = Generator::new();
        let rng = &mut thread_rng();
        let world = generator.generate_world(rng);
        let season = generator.generate_season(rng, &world);

        assert_ne!(season.all_series.len(), 0);
    }
}