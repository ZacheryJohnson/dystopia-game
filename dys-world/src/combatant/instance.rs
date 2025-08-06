use serde::{Deserialize, Serialize};
use ts_rs::TS;
use crate::attribute::attribute_source::AttributeSource;
use crate::attribute::attribute_type::AttributeType;
use crate::attribute::instance::{AttributeInstance, AttributeValueT};
use crate::combatant::limb::Limb;

pub type CombatantInstanceId = u32;

#[derive(Debug, Deserialize, Serialize, TS)]
#[ts(export)]
pub struct CombatantInstance {
    pub id: CombatantInstanceId,
    pub name: String,
    pub limbs: Vec<Limb>,
    pub effect_modifiers: Vec<EffectInstance>,
}

#[derive(Debug, Deserialize, Serialize, TS)]
pub enum EffectDuration {
    NumberOfMatches(u32),
}

#[derive(Debug, Deserialize, Serialize, TS)]
pub struct EffectInstance {
    pub attribute_modifier: Vec<AttributeInstance>,
    pub duration: EffectDuration,
}

impl CombatantInstance {
    pub fn apply_effect(
        &mut self,
        effect: AttributeInstance,
        duration: EffectDuration,
    ) {
        self.effect_modifiers.push(EffectInstance {
            attribute_modifier: vec![effect],
            duration,
        });
    }

    pub fn tick_effects(&mut self) {
        for effect in self.effect_modifiers.iter_mut() {
            match effect.duration {
                EffectDuration::NumberOfMatches(n) => {
                    effect.duration = EffectDuration::NumberOfMatches(if n > 0 {n - 1} else {0});
                },
            }
        }

        // Remove modifiers after expiration
        self.effect_modifiers.retain(|effect| {
            match effect.duration {
                EffectDuration::NumberOfMatches(n) => n > 0,
            }
        });
    }

    pub fn get_attribute_value(&self, attribute_type: &AttributeType) -> Option<AttributeValueT> {
        let temp_effects = self
            .effect_modifiers
            .iter()
            .flat_map(|effect| effect
                .attribute_modifier
                .iter()
                .filter(|inst| inst.attribute_type() == attribute_type)
                .map(|inst| inst.value())
                .collect::<Vec<_>>()
            )
            .collect::<Vec<_>>();

        self
            .limbs
            .iter()
            .filter_map(|limb| limb.attribute_total(attribute_type))
            .chain(temp_effects)
            .fold(None, |acc, attribute_value| Some(acc.unwrap_or_default() + attribute_value))
    }

    /// A combatant's move speed, expressed in units travelable per tick.
    pub fn move_speed(&self) -> f32 {
        let dexterity = self
            .get_attribute_value(&AttributeType::Dexterity)
            .unwrap_or_default();

        // ZJ-TODO: tune this value
        // ZJ-TODO: factor in weight
        dexterity / 50.0
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_effect() {

    }
}