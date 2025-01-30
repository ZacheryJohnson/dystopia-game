use serde::{Deserialize, Serialize};
use crate::attribute::attribute_source::AttributeSource;
use crate::attribute::attribute_type::AttributeType;
use crate::attribute::instance::AttributeValueT;
use crate::combatant::limb::Limb;

pub type CombatantInstanceId = u64;

#[derive(Debug, Deserialize, Serialize)]
pub struct CombatantInstance {
    pub id: CombatantInstanceId,
    pub name: String,
    pub limbs: Vec<Limb>
}

impl CombatantInstance {
    pub fn get_attribute_value(&self, attribute_type: &AttributeType) -> Option<AttributeValueT> {
        self
            .limbs
            .iter()
            .filter_map(|limb| limb.attribute_total(attribute_type))
            .fold(None, |acc, attribute_value| Some(acc.unwrap_or(AttributeValueT::default() + attribute_value)))
    }

    /// A combatant's move speed, expressed in units travelable per tick.
    pub fn move_speed(&self) -> f32 {
        let dexterity = self
            .get_attribute_value(&AttributeType::Dexterity)
            .unwrap_or(AttributeValueT::default());

        // ZJ-TODO: tune this value
        // ZJ-TODO: factor in weight
        dexterity / 50.0
    }
}
