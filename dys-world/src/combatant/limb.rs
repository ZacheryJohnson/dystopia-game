use std::collections::HashMap;
use crate::stat::stat::{Stat, StatType};
use crate::stat::stat_display::StatDisplay;
use crate::stat::stat_source::StatSource;

#[derive(Clone, Debug)]
pub enum LimbType {
    Head,
    Eye,
    Nose,
    Mouth,
    Arm,
    Hand,
    Finger,
    Torso,
    Leg,
    Knee,
    Foot,
    Toe
}

#[derive(Clone, Debug)]
pub enum LimbModifierType {
    Regular,
    Giant,
    Tiny,
    Weak,
    Strong,
    // etc
}

#[derive(Clone, Debug)]
pub enum ModifierAcquisitionMethod {
    Inherent,
    GameInjury,
}

#[derive(Clone, Debug)]
pub struct LimbModifier {
    pub modifier_type: LimbModifierType,
    pub acquisition: ModifierAcquisitionMethod,
    pub stats: Vec<Stat>,
}

impl StatDisplay for LimbModifier {
    fn should_display(&self) -> bool {
        !self.stats.is_empty()
    }
}

impl StatSource for LimbModifier {
    fn stats(&self) -> Vec<Stat> {
        self.stats.clone()
    }
}

#[derive(Clone, Debug)]
pub struct Limb {
    pub limb_type: LimbType,
    pub modifiers: Vec<LimbModifier>,
    pub child_limbs: Vec<Limb>
}

impl StatSource for Limb {
    fn stats(&self) -> Vec<Stat> {
        // ZJ-TODO: refactor good god

        let mut stat_map: HashMap<StatType, f32> = HashMap::new();

        let self_stats: Vec<Stat> = self.modifiers
            .iter()
            .map(|modifier| modifier.stats())
            .fold(vec![], |mut acc, mut stats| { acc.append(&mut stats); acc } );

        let child_stats: Vec<Stat> = self.child_limbs
            .iter()
            .map(|limb| limb.stats())
            .fold(vec![], |mut acc, mut stats| { acc.append(&mut stats); acc } );

        for self_stat in &self_stats {
            match stat_map.get_mut(&self_stat.stat_type) {
                None => {
                    stat_map.insert(self_stat.stat_type.clone(), self_stat.value);
                }
                Some(existing_value) => {
                    *existing_value += self_stat.value;
                }
            }
        }

        for child_stat in &child_stats {
            match stat_map.get_mut(&child_stat.stat_type) {
                None => {
                    stat_map.insert(child_stat.stat_type.clone(), child_stat.value);
                }
                Some(existing_value) => {
                    *existing_value += child_stat.value;
                }
            }
        }

        stat_map
            .iter()
            .map(|(stat_type, stat_value)| Stat { stat_type: stat_type.to_owned(), value: *stat_value } )
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::stat::stat::StatType;
    use super::*;

    #[test]
    fn test() {
        let head = Limb {
            limb_type: LimbType::Head,
            modifiers: vec![
                LimbModifier {
                    modifier_type: LimbModifierType::Regular,
                    acquisition: ModifierAcquisitionMethod::Inherent,
                    stats: vec![Stat {
                        stat_type: StatType::Cognition,
                        value: 2.0,
                    }],
                }
            ],
            child_limbs: vec![
                Limb {
                    limb_type: LimbType::Eye,
                    modifiers: vec![
                        LimbModifier {
                            modifier_type: LimbModifierType::Regular,
                            acquisition: ModifierAcquisitionMethod::Inherent,
                            stats: vec![Stat {
                                stat_type: StatType::Cognition,
                                value: 1.0
                            }],
                        }
                    ],
                    child_limbs: vec![]
                },
            ],
        };

        let expected = vec![Stat {
            stat_type: StatType::Cognition,
            value: 3.0,
        }];

        let actual = head.stats();

        assert_eq!(expected, actual);
    }
}