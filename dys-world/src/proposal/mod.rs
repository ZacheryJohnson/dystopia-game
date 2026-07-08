use serde::{Deserialize, Serialize};
use crate::attribute::instance::AttributeInstance;
use crate::combatant::instance::CombatantInstanceId;

#[derive(Debug, Serialize, Deserialize)]
pub struct Proposal {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub options: Vec<ProposalOption>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProposalOption {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub effects: Vec<ProposalEffect>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ProposalEffect {
    CombatantTemporaryAttributeBonus {
        combatant_instance_id: CombatantInstanceId,
        attribute_instance_bonus: AttributeInstance
    },
}