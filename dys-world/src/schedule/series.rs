use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};

use crate::schedule::serde::serialize_match_instances;
use crate::matches::instance::MatchInstance;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SeriesType {
    /// All matches of the series will be played as normal.
    Normal,

    /// The series will be played until one team reaches the provided number of wins.
    FirstTo(u8),
}

#[derive(Debug, Clone, Serialize)]
pub struct Series {
    #[serde(serialize_with = "serialize_match_instances")]
    pub matches: Vec<Arc<Mutex<MatchInstance>>>,
    pub series_type: SeriesType,
}