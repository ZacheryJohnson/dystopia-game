use serde::{Deserialize, Serialize};

/// Months of the year, naturally sponsored by megacorporations
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Month {
    Arguscorp,
    // ZJ-TODO: add more
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Date(pub Month, pub u32 /* Day */, pub u32 /* Year */);