use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Months of the year, naturally sponsored by megacorporations
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum Month {
    Arguscorp,
    // ZJ-TODO: add more
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Date(pub Month, pub u32 /* Day */, pub u32 /* Year */);