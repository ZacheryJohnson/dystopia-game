use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use crate::attribute::instance::{AttributeInstance, AttributeValueT};
use crate::attribute::attribute_display::AttributeDisplay;
use crate::attribute::attribute_source::AttributeSource;
use crate::attribute::attribute_type::AttributeType;

/// I only learned after developing for months that limbs are specifically jointed appendages.
/// It's not too late to change the name or anything, but I prefer to believe that all the following
/// are actually jointed in this world.
/// That's a little horrifying to think about, but we're not renaming it.
#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(export)]
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

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(export)]
pub enum LimbModifierType {
    Regular,
    Giant,
    Tiny,
    Weak,
    Strong,
    // etc
}

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(export)]
pub enum ModifierAcquisitionMethod {
    Inherent,
    GameInjury,
}

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(export)]
pub struct LimbModifier {
    pub modifier_type: LimbModifierType,
    pub acquisition: ModifierAcquisitionMethod,
    pub attributes: Vec<AttributeInstance>,
}

impl LimbModifier {
    pub fn default_with_attributes(attributes: &[AttributeInstance]) -> LimbModifier {
        LimbModifier {
            modifier_type: LimbModifierType::Regular,
            acquisition: ModifierAcquisitionMethod::Inherent,
            attributes: attributes.to_vec(),
        }
    }
}

impl AttributeDisplay for LimbModifier {
    fn should_display(&self) -> bool {
        !self.attributes.is_empty()
    }
}

impl AttributeSource for LimbModifier {
    fn source_name(&self) -> String {
        format!("{:?} ({:?})", self.modifier_type, self.acquisition)
    }

    fn attribute_total(&self, attribute_type: &AttributeType) -> Option<AttributeValueT> {
        self
            .attributes
            .iter()
            .find(|instance| instance.attribute_type() == attribute_type)
            .map(|instance| instance.value())
    }

    fn attributes(&self) -> Vec<AttributeInstance> {
        self.attributes.clone()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(export)]
pub struct Limb {
    pub limb_type: LimbType,
    pub modifiers: Vec<LimbModifier>,
    pub child_limbs: Vec<Limb>
}

impl AttributeSource for Limb {
    fn source_name(&self) -> String {
        format!("{:?}", self.limb_type)
    }

    fn attribute_total(&self, attribute_type: &AttributeType) -> Option<AttributeValueT> {
        let self_attribute_value = self
            .modifiers
            .iter()
            .filter_map(|modifier| modifier.attribute_total(attribute_type))
            .fold(None, |acc, modifier| Some(acc.unwrap_or_default() + modifier));

        let child_attribute_value = self
            .child_limbs
            .iter()
            .filter_map(|child_limb| child_limb.attribute_total(attribute_type))
            .fold(None, |acc, modifier| Some(acc.unwrap_or_default() + modifier));

        match (self_attribute_value, child_attribute_value) {
            (Some(self_attribute_value), Some(child_attribute_value)) => Some(self_attribute_value + child_attribute_value),
            (Some(self_attribute_value), None) => Some(self_attribute_value),
            (None, Some(child_attribute_value)) => Some(child_attribute_value),
            (None, None) => None
        }
    }
    
    fn attributes(&self) -> Vec<AttributeInstance> {
        let mut stat_map: HashMap<AttributeType, AttributeValueT> = HashMap::new();

        let self_attributes: Vec<AttributeInstance> = self.modifiers
            .iter()
            .map(AttributeSource::attributes)
            .fold(vec![], |mut acc, mut stats| { acc.append(&mut stats); acc } );

        let child_attributes: Vec<AttributeInstance> = self.child_limbs
            .iter()
            .map(AttributeSource::attributes)
            .fold(vec![], |mut acc, mut stats| { acc.append(&mut stats); acc } );

        for attribute in self_attributes.iter().chain(child_attributes.iter()) {
            stat_map
                .entry(attribute.attribute_type().to_owned())
                .and_modify(|existing_attribute_value| *existing_attribute_value += attribute.value())
                .or_insert(attribute.value());
        }

        stat_map
            .iter()
            .map(|(attribute_type, value)| AttributeInstance::new(attribute_type.to_owned(), *value))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::attribute::attribute_type::AttributeType;
    use super::*;

    #[test]
    fn get_all_attributes() {
        let head = Limb {
            limb_type: LimbType::Head,
            modifiers: vec![
                LimbModifier {
                    modifier_type: LimbModifierType::Regular,
                    acquisition: ModifierAcquisitionMethod::Inherent,
                    attributes: vec![AttributeInstance::new(AttributeType::Cognition, 2.0)],
                }
            ],
            child_limbs: vec![
                Limb {
                    limb_type: LimbType::Eye,
                    modifiers: vec![
                        LimbModifier {
                            modifier_type: LimbModifierType::Regular,
                            acquisition: ModifierAcquisitionMethod::Inherent,
                            attributes: vec![AttributeInstance::new(AttributeType::Cognition, 1.0)],
                        }
                    ],
                    child_limbs: vec![]
                },
            ],
        };

        let expected = vec![AttributeInstance::new(AttributeType::Cognition, 3.0)];

        let actual = head.attributes();

        assert_eq!(expected, actual);
    }

    #[test]
    fn get_specific_attribute_exists() {
        let head = Limb {
            limb_type: LimbType::Head,
            modifiers: vec![
                LimbModifier {
                    modifier_type: LimbModifierType::Regular,
                    acquisition: ModifierAcquisitionMethod::Inherent,
                    attributes: vec![AttributeInstance::new(AttributeType::Cognition, 2.0)],
                }
            ],
            child_limbs: vec![
                Limb {
                    limb_type: LimbType::Eye,
                    modifiers: vec![
                        LimbModifier {
                            modifier_type: LimbModifierType::Regular,
                            acquisition: ModifierAcquisitionMethod::Inherent,
                            attributes: vec![AttributeInstance::new(AttributeType::Cognition, 1.0)],
                        }
                    ],
                    child_limbs: vec![]
                },
            ],
        };

        let expected_instance = AttributeInstance::new(AttributeType::Cognition, 3.0);

        let actual_value = head.attribute_total(&AttributeType::Cognition);
        assert!(actual_value.is_some());
        assert_eq!(expected_instance.value(), actual_value.unwrap());
    }

    #[test]
    fn get_specific_attribute_not_exists() {
        let head = Limb {
            limb_type: LimbType::Head,
            modifiers: vec![
                LimbModifier {
                    modifier_type: LimbModifierType::Regular,
                    acquisition: ModifierAcquisitionMethod::Inherent,
                    attributes: vec![AttributeInstance::new(AttributeType::Cognition, 2.0)],
                }
            ],
            child_limbs: vec![
                Limb {
                    limb_type: LimbType::Eye,
                    modifiers: vec![
                        LimbModifier {
                            modifier_type: LimbModifierType::Regular,
                            acquisition: ModifierAcquisitionMethod::Inherent,
                            attributes: vec![AttributeInstance::new(AttributeType::Cognition, 1.0)],
                        }
                    ],
                    child_limbs: vec![]
                },
            ],
        };

        let actual_value = head.attribute_total(&AttributeType::Strength);
        assert!(actual_value.is_none());
    }
}