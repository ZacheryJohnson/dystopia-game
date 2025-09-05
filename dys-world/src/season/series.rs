use std::collections::BTreeMap;
use std::sync::{Mutex, Weak};
use serde::{Deserialize, Serialize};

use crate::games::instance::GameInstance;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SeriesType {
    /// All games of the series will be played as normal.
    Normal,

    /// The series will be played until one team reaches the provided number of wins.
    FirstTo(u8),
}

/// Monotonically increasing value, where games are played incrementally by game index
pub type SeriesGameIndex = u8;

/// Series are collections of games that are played in order against the same opponent.
/// This is commonly used in playoff formats,
/// but in sports like baseball where many games are played over a season,
/// it makes more logistical sense to play those games in one location to reduce travel costs.
#[derive(Debug, Clone, Serialize)]
pub struct Series {
    /// Weak references to games that exist in this series.
    /// The games are authoritatively owned elsewhere, so we must validate they exist before use.
    games: BTreeMap<SeriesGameIndex, Weak<Mutex<GameInstance>>>,
    series_type: SeriesType,
}

impl Series {
    /// Creates a series from an iterable,
    /// assuming that the games are in the order the series should be played.
    pub fn from_ordered_games(
        games: &[Weak<Mutex<GameInstance>>],
        series_type: SeriesType
    ) -> Series {
        let games = games
            .iter()
            .enumerate()
            .map(|(idx, game)| (idx as SeriesGameIndex, game.to_owned()))
            .collect();

        Series {
            games,
            series_type,
        }
    }

    /// Returns the games of the series in order.
    pub fn games(&self) -> Vec<Weak<Mutex<GameInstance>>> {
        self
            .games
            .values()
            .map(|game| game.to_owned())
            .collect()
    }

    pub fn series_type(&self) -> SeriesType {
        self.series_type.clone()
    }
}