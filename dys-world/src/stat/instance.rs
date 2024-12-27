#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum StatType {
    Strength,
    Dexterity,
    Cognition,
    Reaction,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StatInstance {
    pub stat_type: StatType,
    pub value: f32,
}

impl From<StatInstance> for (StatType, f32) {
    fn from(val: StatInstance) -> Self {
        (val.stat_type, val.value)
    }
}