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

impl Into<(StatType, f32)> for Stat {
    fn into(self) -> (StatType, f32) {
        (self.stat_type, self.value)
    }
}