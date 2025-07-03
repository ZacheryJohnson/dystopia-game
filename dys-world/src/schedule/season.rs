use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use chrono::{Duration, Timelike};
use crate::matches::instance::{MatchInstance, MatchInstanceId};
use crate::schedule::calendar::Date;
use crate::schedule::series::Series;

#[derive(Debug, Clone)]
pub struct Season {
    pub all_series: Vec<Series>,
    pub simulation_timings: HashMap<MatchInstanceId, chrono::DateTime<chrono::Utc>>,
}

impl Season {
    pub fn new(all_series: Vec<Series>) -> Self {
        let mut simulation_timings = HashMap::new();

        // Schedule matches every 15 minutes on the dot for now
        let now_utc = chrono::Utc::now();
        let second_adjustment = 60 - now_utc.second() as i64 % 60;
        let second_adjusted_utc = now_utc + Duration::seconds(second_adjustment);

        let minute_adjustment = 15 - second_adjusted_utc.minute() as i64 % 15;

        let first_match_time_utc = second_adjusted_utc + Duration::minutes(minute_adjustment);

        // ZJ-TODO: refactor
        #[allow(unused_assignments)]
        let mut next_match_time_utc = first_match_time_utc;

        for series in &all_series {
            for match_instance in &series.matches {
                let days_since_first = match_instance.lock().unwrap().date.1 - 1;
                next_match_time_utc = first_match_time_utc + Duration::minutes(15 * days_since_first as i64);

                simulation_timings.insert(
                    match_instance.lock().unwrap().match_id,
                    next_match_time_utc
                );
            }
        }

        Season {
            all_series,
            simulation_timings,
        }
    }

    pub fn matches_on_date(&self, date: &Date) -> Vec<Arc<Mutex<MatchInstance>>> {
        self
            .all_series
            .iter()
            .flat_map(|series| series.matches.clone())
            .filter(|match_| match_.lock().unwrap().date == *date)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::matches::instance::MatchInstanceId;
    use crate::schedule::calendar::Month;
    use crate::schedule::series::SeriesType;
    use crate::team::instance::TeamInstance;
    use super::*;

    #[test]
    fn get_matches_on_date() {
        let make_match_with_date = |match_id: MatchInstanceId, date: &Date| -> Arc<Mutex<MatchInstance>> {
            Arc::new(Mutex::new(MatchInstance {
                match_id,
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
                    matches: vec![
                        make_match_with_date(1, &Date(Month::Arguscorp, 1, 10000)),
                        make_match_with_date(2, &Date(Month::Arguscorp, 1, 10000)),
                        make_match_with_date(3, &Date(Month::Arguscorp, 2, 10000)),
                        make_match_with_date(4, &Date(Month::Arguscorp, 3, 10000)),
                        make_match_with_date(5, &Date(Month::Arguscorp, 4, 10000)),
                        make_match_with_date(6, &Date(Month::Arguscorp, 1, 10000)),
                    ],
                    series_type: SeriesType::Normal,
                },
            ],
        );

        assert_eq!(season.matches_on_date(&Date(Month::Arguscorp, 1, 10000)).len(), 3);
        assert_eq!(season.matches_on_date(&Date(Month::Arguscorp, 2, 10000)).len(), 1);
        assert_eq!(season.matches_on_date(&Date(Month::Arguscorp, 3, 10000)).len(), 1);
        assert_eq!(season.matches_on_date(&Date(Month::Arguscorp, 4, 10000)).len(), 1);
    }
}