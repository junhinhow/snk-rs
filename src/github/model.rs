//! The data model of a contribution calendar.

use chrono::NaiveDate;

/// A single day of the contribution calendar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContributionDay {
    /// The day itself.
    pub date: NaiveDate,
    /// How many contributions happened on that day.
    ///
    /// Exact when the calendar is fetched with a token; an estimate derived
    /// from the level when the public page is scraped.
    pub count: i64,
    /// Contribution level, from `0` (nothing) to `4` (a lot).
    pub level: u8,
}

/// A whole contribution calendar.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ContributionCalendar {
    /// Every day of the period, ordered by date.
    pub days: Vec<ContributionDay>,
    /// Total number of contributions in the period.
    pub total: i64,
}

impl ContributionCalendar {
    /// `true` when the calendar has no days at all.
    pub fn is_empty(&self) -> bool {
        self.days.is_empty()
    }

    /// Number of days that hold at least one contribution.
    pub fn active_days(&self) -> usize {
        self.days.iter().filter(|day| day.level > 0).count()
    }

    /// Sorts the days by date and recomputes the total.
    pub fn normalize(&mut self) {
        self.days.sort_by_key(|day| day.date);
        self.total = self.days.iter().map(|day| day.count).sum();
    }
}

/// Maps a contribution level to the amount of contributions it represents.
///
/// GitHub's legend is `0`, `1-3`, `4-6`, `7-9` and `10+`; the midpoint of each
/// range is used as an estimate.
pub fn estimate_count(level: u8) -> i64 {
    match level {
        0 => 0,
        1 => 2,
        2 => 5,
        3 => 8,
        _ => 10,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn estimates_follow_the_github_legend() {
        assert_eq!(estimate_count(0), 0);
        assert_eq!(estimate_count(1), 2);
        assert_eq!(estimate_count(4), 10);
    }

    #[test]
    fn normalize_sorts_and_totals() {
        let mut calendar = ContributionCalendar {
            days: vec![
                ContributionDay {
                    date: NaiveDate::parse_from_str("2024-02-01", "%Y-%m-%d").unwrap(),
                    count: 4,
                    level: 2,
                },
                ContributionDay {
                    date: NaiveDate::parse_from_str("2024-01-01", "%Y-%m-%d").unwrap(),
                    count: 6,
                    level: 3,
                },
            ],
            total: 0,
        };
        calendar.normalize();
        assert_eq!(calendar.total, 10);
        assert!(calendar.days[0].date.to_string().starts_with("2024-01"));
        assert_eq!(calendar.active_days(), 2);
    }
}
