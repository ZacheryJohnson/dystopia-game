use std::sync::{Arc, Mutex};
use crate::matches::instance::MatchInstance;
use crate::schedule::calendar::Date;
use crate::schedule::series::Series;

#[derive(Debug)]
pub struct Season {
    pub all_series: Vec<Series>,
}

impl Season {
    pub fn matches_on_date(&self, date: Date) -> Vec<Arc<Mutex<MatchInstance>>> {
        self
            .all_series
            .iter()
            .flat_map(|series| series.matches.clone())
            .filter(|match_| match_.lock().unwrap().date == date)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::arena::Arena;
    use crate::matches::instance::MatchInstanceId;
    use crate::schedule::calendar::Month;
    use crate::schedule::series::SeriesType;
    use crate::team::instance::TeamInstance;
    use super::*;

    #[test]
    fn get_matches_on_date() {
        let make_match_with_date = |match_id: MatchInstanceId, date: Date| -> Arc<Mutex<MatchInstance>> {
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
                arena: Arc::new(Mutex::new(Arena::new_with_testing_defaults())),
                date,
            }))
        };

        let season = Season {
            all_series: vec![
                Series {
                    matches: vec![
                        make_match_with_date(1, Date(Month::Arguscorp, 1, 10000)),
                        make_match_with_date(2, Date(Month::Arguscorp, 1, 10000)),
                        make_match_with_date(3, Date(Month::Arguscorp, 2, 10000)),
                        make_match_with_date(4, Date(Month::Arguscorp, 3, 10000)),
                        make_match_with_date(5, Date(Month::Arguscorp, 4, 10000)),
                        make_match_with_date(6, Date(Month::Arguscorp, 1, 10000)),
                    ],
                    series_type: SeriesType::Normal,
                }
            ]
        };

        assert_eq!(season.matches_on_date(Date(Month::Arguscorp, 1, 10000)).len(), 3);
        assert_eq!(season.matches_on_date(Date(Month::Arguscorp, 2, 10000)).len(), 1);
        assert_eq!(season.matches_on_date(Date(Month::Arguscorp, 3, 10000)).len(), 1);
        assert_eq!(season.matches_on_date(Date(Month::Arguscorp, 4, 10000)).len(), 1);
    }
}