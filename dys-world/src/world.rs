use std::sync::{Arc, Mutex};
use serde::Serialize;
use crate::{
    combatant::instance::CombatantInstance,
    serde::{serialize_combatants, serialize_teams},
    team::instance::TeamInstance,
};

#[derive(Clone, Serialize)]
pub struct World {
    #[serde(serialize_with = "serialize_combatants")]
    pub combatants: Vec<Arc<Mutex<CombatantInstance>>>,

    #[serde(serialize_with = "serialize_teams")]
    pub teams: Vec<Arc<Mutex<TeamInstance>>>,
}

#[cfg(test)]
mod tests {
    use crate::attribute::attribute_type::AttributeType;
    use crate::attribute::instance::AttributeInstance;
    use crate::combatant::limb::{Limb, LimbModifier, LimbType};
    use super::*;

    #[test]
    fn serialize_into_deserialize() {
        let combatants = vec![
            Arc::new(Mutex::new(
                CombatantInstance {
                    id: 1,
                    name: String::from("Combatant 1"),
                    limbs: vec![
                        Limb {
                            limb_type: LimbType::Head,
                            modifiers: vec![
                                LimbModifier::default_with_attributes(&[
                                    AttributeInstance::new(
                                        AttributeType::Cognition,
                                        30.0
                                    )
                                ])
                            ],
                            child_limbs: vec![],
                        }
                    ],
                }
            )),
            Arc::new(Mutex::new(
                CombatantInstance {
                    id: 2,
                    name: String::from("Combatant 2"),
                    limbs: vec![
                        Limb {
                            limb_type: LimbType::Torso,
                            modifiers: vec![
                                LimbModifier::default_with_attributes(&[
                                    AttributeInstance::new(
                                        AttributeType::Constitution,
                                        20.0
                                    )
                                ])
                            ],
                            child_limbs: vec![],
                        }
                    ],
                }
            )),
        ];

        let teams = vec![
            Arc::new(Mutex::new(
                TeamInstance {
                    id: 1,
                    name: "Team 1".to_string(),
                    combatants: vec![
                        combatants[0].clone(),
                    ],
                }
            )),
            Arc::new(Mutex::new(
                TeamInstance {
                    id: 2,
                    name: "Team 2".to_string(),
                    combatants: vec![
                        combatants[1].clone(),
                    ],
                }
            )),
        ];

        let world = World {
            combatants,
            teams
        };

        let serialized = serde_json::to_string(&world).unwrap();

        let deserialized: World = serde_json::from_str(&serialized).unwrap();

        assert_eq!(world.combatants.len(), deserialized.combatants.len());
        assert_eq!(world.teams.len(), deserialized.teams.len());
    }
}