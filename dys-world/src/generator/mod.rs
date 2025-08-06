use std::collections::{BTreeMap, HashMap};
use std::ops::RangeInclusive;
use std::sync::{Arc, Mutex};
use rand::prelude::IteratorRandom;
use rand::Rng;
use rand_distr::{Distribution, Normal};
use crate::combatant::instance::{CombatantInstance, CombatantInstanceId};
use crate::combatant::limb::{Limb, LimbModifier, LimbType};
use crate::attribute::instance::AttributeInstance;
use crate::attribute::attribute_type::AttributeType;
use crate::team::instance::{TeamInstance, TeamInstanceId};
use crate::world::World;
use crate::games::instance::GameInstance;
use crate::proposal::{Proposal, ProposalEffect, ProposalOption};
use crate::schedule::calendar::{Date, Month};
use crate::season::season::{GamesMapT, ScheduleMapT, Season};
use crate::season::series::{Series, SeriesType};

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

    fn generate_combatant(&self, id: u32, rng: &mut impl Rng) -> CombatantInstance {
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
    
    pub fn generate_combatants(
        &self,
        count: u32,
        rng: &mut impl Rng
    ) -> HashMap<CombatantInstanceId, Arc<Mutex<CombatantInstance>>> {
        let mut combatants = HashMap::new();

        for i in 0..count {
            let new_combatant = self.generate_combatant(i, rng);
            combatants.insert(new_combatant.id, Arc::new(Mutex::new(new_combatant)));
        }
    
        combatants
    }

    pub fn generate_world(&self, rng: &mut impl Rng) -> World {
        // ZJ-TODO: this should be config driven
        let number_of_teams = 4;
        let players_per_team = 5;
        let total_combatants_to_generate = number_of_teams * players_per_team;

        let combatants = self.generate_combatants(total_combatants_to_generate, rng);
        let sorted_combatants = BTreeMap::from_iter(&combatants);

        let teams = sorted_combatants
            .values()
            .collect::<Vec<_>>()
            .chunks(players_per_team as usize)
            .enumerate()
            .map(|(id, combatants)| TeamInstance {
                id: id as u32 + 1,
                name: self.team_names[id].clone(),
                combatants: combatants.iter().map(|arc| (**arc).to_owned()).collect::<Vec<_>>(),
            })
            .map(|team| (team.id, Arc::new(Mutex::new(team))))
            .collect();

        let season = self.generate_season(rng, &teams);
            
        World {
            combatants,
            teams,
            season
        }
    }

    pub fn generate_season(
        &self,
        rng: &mut impl Rng,
        teams: &HashMap<TeamInstanceId, Arc<Mutex<TeamInstance>>>,
    ) -> Season {
        // ZJ-TODO: I'd love for this to be more interesting
        // For now, just do a simple round-robin of 3 game series

        const SERIES_COUNT: u16 = 33;
        const SERIES_LEN_RANGE: RangeInclusive<u16> = 3..=3;
        let series_len = SERIES_LEN_RANGE.choose(rng).unwrap();

        // Create a first-pass schedule: we'll make changes + validate after

        let mut games = GamesMapT::new();
        let mut schedule = ScheduleMapT::new();
        let mut series = vec![];

        let teams = teams.values().collect::<Vec<_>>();
        let fixed_team = teams.first().unwrap().to_owned();
        let mut rotating_teams = Vec::from_iter(
            teams
                .iter()
                .skip(1)
                .map(|arc| (*arc).to_owned())
        );

        let mut game_id = 0;
        let mut date = Date(Month::Arguscorp, 1, 10000);
        for series_idx in 0..SERIES_COUNT {
            let swap_fixed_matchup = series_idx % 2 >= 1;
            let swap_alt_matchup = series_idx % 4 >= 2;

            let fixed_opponent = rotating_teams.pop().unwrap().to_owned();
            let alt_opponent_1 = rotating_teams.pop().unwrap().to_owned();
            let alt_opponent_2 = rotating_teams.pop().unwrap().to_owned();

            let mut fixed_series_games = vec![];
            let mut alt_series_games = vec![];
            for _ in 0..series_len {
                {
                    game_id += 1;
                    let game = Arc::new(Mutex::new(GameInstance {
                        game_id,
                        away_team: if swap_fixed_matchup { fixed_team.clone() } else { fixed_opponent.clone() },
                        home_team: if swap_fixed_matchup { fixed_opponent.clone() } else { fixed_team.clone() },
                        // arena: Arc::new(Mutex::new(Arena::new_with_testing_defaults())), // ZJ-TODO
                        arena_id: 0,
                        date: date.clone(),
                    }));

                    let game_reference = Arc::downgrade(&game);
                    fixed_series_games.push(game_reference.clone());

                    games.insert(game_id, game);
                    schedule.entry(date.clone()).or_default().push(game_reference);
                }
                {
                    game_id += 1;
                    let game = Arc::new(Mutex::new(GameInstance {
                        game_id,
                        away_team: if swap_alt_matchup { alt_opponent_1.clone() } else { alt_opponent_2.clone() },
                        home_team: if swap_alt_matchup { alt_opponent_2.clone() } else { alt_opponent_1.clone() },
                        // arena: Arc::new(Mutex::new(Arena::new_with_testing_defaults())), // ZJ-TODO
                        arena_id: 0,
                        date: date.clone(),
                    }));

                    let game_reference = Arc::downgrade(&game);
                    alt_series_games.push(game_reference.clone());

                    games.insert(game_id, game);
                    schedule.entry(date.clone()).or_default().push(game_reference);
                }

                date.1 += 1;
            }

            let fixed_series = Series::from_ordered_games(
                &fixed_series_games,
                SeriesType::Normal,
            );

            let alt_series = Series::from_ordered_games(
                &alt_series_games,
                SeriesType::Normal,
            );

            series.push(fixed_series);
            series.push(alt_series);

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

        Season::new(games, schedule, series)
    }

    pub fn generate_proposals(
        &self,
        _: &mut impl Rng,
        world: &World
    ) -> Vec<Proposal> {
        let mut proposals = vec![];

        for (proposal_id, (_, team)) in world.teams.iter().enumerate() {
            // enumerate is zero-indexed and usize, but we expect one-indexed and u64 elsewhere
            let proposal_id = (proposal_id + 1) as u64;

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
    use rand::rng;
    use super::*;

    #[test]
    fn generate_season() {
        let generator = Generator::new();
        let rng = &mut rng();
        let world = generator.generate_world(rng);
        let season = generator.generate_season(rng, &world.teams);

        assert_ne!(season.games().len(), 0);
    }
}