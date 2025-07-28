use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use chrono::{Duration, Timelike};
use crate::games::instance::{GameInstance, GameInstanceId};
use crate::schedule::calendar::Date;
use crate::season::series::Series;

/// Seasons are the collection of games that will be played
/// and the scheduling of those games.
#[derive(Debug, Clone)]
pub struct Season {
    games: Vec<Arc<Mutex<GameInstance>>>,
    all_series: Vec<Series>,
    pub simulation_timings: HashMap<GameInstanceId, chrono::DateTime<chrono::Utc>>,
}

impl Season {
    pub fn new(all_series: Vec<Series>) -> Self {
        let mut simulation_timings = HashMap::new();

        // Schedule matches every 15 minutes on the dot for now
        let now_utc = chrono::Utc::now();
        let second_adjustment = 60 - now_utc.second() as i64 % 60;
        let second_adjusted_utc = now_utc + Duration::seconds(second_adjustment);

        let minute_adjustment = 15 - second_adjusted_utc.minute() as i64 % 15;

        let first_game_time_utc = second_adjusted_utc + Duration::minutes(minute_adjustment);

        // ZJ-TODO: refactor
        #[allow(unused_assignments)]
        let mut next_game_time_utc = first_game_time_utc;

        for series in &all_series {
            for (_, game_instance) in &series.games {
                let days_since_first = game_instance.lock().unwrap().date.1 - 1;
                next_game_time_utc = first_game_time_utc + Duration::minutes(15 * days_since_first as i64);

                simulation_timings.insert(
                    game_instance.lock().unwrap().game_id,
                    next_game_time_utc
                );
            }
        }

        let games = all_series
            .iter()
            .flat_map(|series| series.games.to_owned())
            .map(|(_, game)| game)
            .collect();

        Season {
            games,
            all_series,
            simulation_timings,
        }
    }

    pub fn games(&self) -> &Vec<Arc<Mutex<GameInstance>>> {
        &self.games
    }

    pub fn series(&self) -> &Vec<Series> {
        &self.all_series
    }

    pub fn games_on_date(&self, date: &Date) -> Vec<Arc<Mutex<GameInstance>>> {
        self
            .games
            .iter()
            .filter(|game| game.lock().unwrap().date == *date)
            .map(|game| game.to_owned())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
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

        let season = Season::new(
            vec![
                Series {
                    games: BTreeMap::from([
                        (1, make_game_with_date(1, &Date(Month::Arguscorp, 1, 10000))),
                        (2, make_game_with_date(2, &Date(Month::Arguscorp, 1, 10000))),
                        (3, make_game_with_date(3, &Date(Month::Arguscorp, 2, 10000))),
                        (4, make_game_with_date(4, &Date(Month::Arguscorp, 3, 10000))),
                        (5, make_game_with_date(5, &Date(Month::Arguscorp, 4, 10000))),
                        (6, make_game_with_date(6, &Date(Month::Arguscorp, 1, 10000))),
                    ]),
                    series_type: SeriesType::Normal,
                },
            ],
        );

        assert_eq!(season.games_on_date(&Date(Month::Arguscorp, 1, 10000)).len(), 3);
        assert_eq!(season.games_on_date(&Date(Month::Arguscorp, 2, 10000)).len(), 1);
        assert_eq!(season.games_on_date(&Date(Month::Arguscorp, 3, 10000)).len(), 1);
        assert_eq!(season.games_on_date(&Date(Month::Arguscorp, 4, 10000)).len(), 1);
    }
}