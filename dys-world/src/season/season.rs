use std::collections::HashMap;
use std::sync::{Arc, Mutex, Weak};
use serde::Serialize;
use ts_rs::TS;
use crate::games::instance::{GameInstance, GameInstanceId};
use crate::schedule::calendar::Date;
use crate::season::series::Series;

pub type GamesMapT = HashMap<GameInstanceId, Arc<Mutex<GameInstance>>>;
pub type ScheduleMapT = HashMap<Date, Vec<Weak<Mutex<GameInstance>>>>;

/// Seasons are the collection of games that will be played
/// and the scheduling of those games.
#[derive(Debug, Clone, Serialize, TS)]
pub struct Season {
    /// This is considered the authoritative source-of-truth for all games.
    #[serde(skip_serializing)]
    #[ts(skip)]
    games: GamesMapT,
    #[serde(skip_serializing)]
    #[ts(skip)]
    schedule: ScheduleMapT,
    // ZJ-TODO: restore serializing
    #[serde(skip_serializing)]
    #[ts(skip)]
    series: Vec<Series>,
}

impl Season {
    pub fn new(
        games: GamesMapT,
        schedule: ScheduleMapT,
        series: Vec<Series>
    ) -> Self {
        Season {
            games,
            schedule,
            series,
        }
    }

    pub fn games(&self) -> Vec<Weak<Mutex<GameInstance>>> {
        self
            .games
            .values()
            .map(Arc::downgrade)
            .collect()
    }

    pub fn series(&self) -> &Vec<Series> {
        &self.series
    }

    pub fn games_on_date(&self, date: &Date) -> Vec<Weak<Mutex<GameInstance>>> {
        self
            .schedule
            .get(date)
            .unwrap_or(&vec![])
            .iter()
            .map(|weak_ref| weak_ref.to_owned())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::games::instance::GameInstanceId;
    use crate::schedule::calendar::Month;
    use crate::season::series::SeriesType;
    use crate::team::instance::TeamInstance;
    use super::*;

    #[test]
    fn get_matches_on_date() {
        let make_game_with_date = |game_id: GameInstanceId, date: &Date| -> Arc<Mutex<GameInstance>> {
            Arc::new(Mutex::new(GameInstance {
                game_id,
                away_team: Arc::new(Mutex::new(TeamInstance {
                    id: 1,
                    name: String::new(),
                    combatants: vec![],
                })),
                home_team: Arc::new(Mutex::new(TeamInstance {
                    id: 2,
                    name: String::new(),
                    combatants: vec![],
                })),
                // arena: Arc::new(Mutex::new(Arena::new_with_testing_defaults())),
                arena_id: 0,
                date: date.to_owned(),
            }))
        };

        let games: GamesMapT = HashMap::from([
            (1, make_game_with_date(1, &Date::new(Month::Arguscorp, 1, 10000))),
            (2, make_game_with_date(2, &Date::new(Month::Arguscorp, 1, 10000))),
            (3, make_game_with_date(3, &Date::new(Month::Arguscorp, 2, 10000))),
            (4, make_game_with_date(4, &Date::new(Month::Arguscorp, 3, 10000))),
            (5, make_game_with_date(5, &Date::new(Month::Arguscorp, 4, 10000))),
            (6, make_game_with_date(6, &Date::new(Month::Arguscorp, 1, 10000))),
        ]);

        let games_ref: Vec<_> = games
            .values()
            .map(|game| Arc::downgrade(game))
            .collect();

        let mut schedule = ScheduleMapT::new();
        for (_, game) in &games {
            let date = game.lock().unwrap().date.to_owned();
            schedule
                .entry(date)
                .or_default()
                .push(Arc::downgrade(game));
        }

        let season = Season::new(
            games,
            schedule,
            vec![
                Series::from_ordered_games(
                    &games_ref,
                    SeriesType::Normal,
                ),
            ],
        );

        assert_eq!(season.games_on_date(&Date::new(Month::Arguscorp, 1, 10000)).len(), 3);
        assert_eq!(season.games_on_date(&Date::new(Month::Arguscorp, 2, 10000)).len(), 1);
        assert_eq!(season.games_on_date(&Date::new(Month::Arguscorp, 3, 10000)).len(), 1);
        assert_eq!(season.games_on_date(&Date::new(Month::Arguscorp, 4, 10000)).len(), 1);
    }
}