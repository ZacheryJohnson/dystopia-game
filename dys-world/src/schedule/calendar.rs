use std::cmp::Ordering;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Months of the year, naturally sponsored by megacorporations
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum Month {
    Arguscorp,
    // ZJ-TODO: add more
}

impl PartialOrd for Month {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let mut found_self;
        let mut found_other;

        for month in Month::in_order() {
            found_self = self == month;
            found_other = other == month;

            if found_self && found_other {
                return Some(Ordering::Equal);
            } else if found_self {
                return Some(Ordering::Less);
            } else if found_other {
                return Some(Ordering::Greater);
            }
        }

        panic!("unable to find month, which breaks PartialOrd + Ord")
    }
}

impl Ord for Month {
    fn cmp(&self, other: &Self) -> Ordering {
        // We will never return None when partial comparing, so just unwrap that value
        self.partial_cmp(other).unwrap()
    }
}

impl Month {
    pub fn days(&self) -> u32 {
        match self {
            Month::Arguscorp => 30,
        }
    }

    fn in_order() -> &'static [Month] {
        &[
            Month::Arguscorp,
        ]
    }

    pub fn id(&self) -> u32 {
        match self {
            Month::Arguscorp => 1,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Date(
    Month,
    u32 /* Day */,
    u32 /* Year */
);

impl Date {
    pub fn new(month: Month, day: u32, year: u32) -> Date {
        Date(month, day, year)
    }

    pub fn month(&self) -> Month {
        self.0.to_owned()
    }

    pub fn month_mut(&mut self) -> &mut Month {
        &mut self.0
    }

    pub fn day(&self) -> u32 {
        self.1
    }

    pub fn day_mut(&mut self) -> &mut u32 {
        &mut self.1
    }

    pub fn year(&self) -> u32 {
        self.2
    }

    pub fn year_mut(&mut self) -> &mut u32 {
        &mut self.2
    }

    pub fn as_monotonic(&self) -> u32 {
        let mut day_count = 0;
        for month in Month::in_order() {
            if self.month() == *month {
                day_count += self.day();
                break;
            } else {
                day_count += month.days();
            }
        }

        day_count
    }
}