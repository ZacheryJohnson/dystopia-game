use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use crate::matches::instance::MatchInstance;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SeriesType {
    /// All matches of the series will be played as normal.
    Normal,

    /// The series will be played until one team reaches the provided number of wins.
    FirstTo(u8),
}

#[derive(Debug, Clone)]
pub struct Series {
    pub matches: Vec<Arc<Mutex<MatchInstance>>>,
    pub series_type: SeriesType,
}