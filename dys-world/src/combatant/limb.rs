use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::attribute::instance::AttributeInstance;
use crate::attribute::attribute_display::AttributeDisplay;
use crate::attribute::attribute_source::AttributeSource;
use crate::attribute::attribute_type::AttributeType;

/// I only learned after developing for months that limbs are specifically jointed appendages.
/// It's not too late to change the name or anything, but I prefer to believe that all the following
/// are actually jointed in this world.
/// That's a little horrifying to think about, but we're not renaming it.
#[derive(Clone, Debug, Deserialize, Serialize)]
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum LimbModifierType {
    Regular,
    Giant,
    Tiny,
    Weak,
    Strong,
    // etc
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ModifierAcquisitionMethod {
    Inherent,
    GameInjury,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LimbModifier {
    pub modifier_type: LimbModifierType,
    pub acquisition: ModifierAcquisitionMethod,
    pub stats: Vec<AttributeInstance>,
}

impl LimbModifier {
    pub fn default_with_attributes(attributes: &[AttributeInstance]) -> LimbModifier {
        LimbModifier {
            modifier_type: LimbModifierType::Regular,
            acquisition: ModifierAcquisitionMethod::Inherent,
            stats: attributes.to_vec(),
        }
    }
}

impl AttributeDisplay for LimbModifier {
    fn should_display(&self) -> bool {
        !self.stats.is_empty()
    }
}

impl AttributeSource for LimbModifier {
    fn stats(&self) -> Vec<AttributeInstance> {
        self.stats.clone()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Limb {
    pub limb_type: LimbType,
    pub modifiers: Vec<LimbModifier>,
    pub child_limbs: Vec<Limb>
}

impl AttributeSource for Limb {
    fn stats(&self) -> Vec<AttributeInstance> {
        // ZJ-TODO: refactor good god

        let mut stat_map: HashMap<AttributeType, f32> = HashMap::new();

        let self_stats: Vec<AttributeInstance> = self.modifiers
            .iter()
            .map(|modifier| modifier.stats())
            .fold(vec![], |mut acc, mut stats| { acc.append(&mut stats); acc } );

        let child_stats: Vec<AttributeInstance> = self.child_limbs
            .iter()
            .map(|limb| limb.stats())
            .fold(vec![], |mut acc, mut stats| { acc.append(&mut stats); acc } );

        for self_stat in &self_stats {
            match stat_map.get_mut(&self_stat.attribute_type()) {
                None => {
                    stat_map.insert(self_stat.attribute_type().clone(), self_stat.value());
                }
                Some(existing_value) => {
                    *existing_value += self_stat.value();
                }
            }
        }

        for child_stat in &child_stats {
            match stat_map.get_mut(&child_stat.attribute_type()) {
                None => {
                    stat_map.insert(child_stat.attribute_type().clone(), child_stat.value());
                }
                Some(existing_value) => {
                    *existing_value += child_stat.value();
                }
            }
        }

        stat_map
            .iter()
            .map(|(attribute_type, value)| AttributeInstance::new(attribute_type.to_owned(), *value) )
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::attribute::attribute_type::AttributeType;
    use super::*;

    #[test]
    fn test() {
        let head = Limb {
            limb_type: LimbType::Head,
            modifiers: vec![
                LimbModifier {
                    modifier_type: LimbModifierType::Regular,
                    acquisition: ModifierAcquisitionMethod::Inherent,
                    stats: vec![AttributeInstance::new(AttributeType::Cognition, 2.0)],
                }
            ],
            child_limbs: vec![
                Limb {
                    limb_type: LimbType::Eye,
                    modifiers: vec![
                        LimbModifier {
                            modifier_type: LimbModifierType::Regular,
                            acquisition: ModifierAcquisitionMethod::Inherent,
                            stats: vec![AttributeInstance::new(AttributeType::Cognition, 1.0)],
                        }
                    ],
                    child_limbs: vec![]
                },
            ],
        };

        let expected = vec![AttributeInstance::new(AttributeType::Cognition, 3.0)];

        let actual = head.stats();

        assert_eq!(expected, actual);
    }
}