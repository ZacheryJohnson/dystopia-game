#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum StatType {
    Strength,
    Dexterity,
    Cognition,
    Reaction,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Stat {
    pub stat_type: StatType,
    pub value: f32,
}

impl From<Stat> for (StatType, f32) {
    fn from(val: Stat) -> Self {
        (val.stat_type, val.value)
    }
}