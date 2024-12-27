#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum StatType {
    Strength,
    Dexterity,
    Cognition,
    Reaction,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StatDefinition {
    pub stat_type: StatType,
    pub value: f32,
}

impl From<StatDefinition> for (StatType, f32) {
    fn from(val: StatDefinition) -> Self {
        (val.stat_type, val.value)
    }
}