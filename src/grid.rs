//! The contribution grid: a matrix of days laid out like GitHub's calendar.

use chrono::{Datelike, Duration, NaiveDate, Weekday};

use crate::error::{Error, Result};
use crate::github::model::ContributionCalendar;

/// A rectangular grid of days, seven rows (Monday..Sunday) by N columns (weeks).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grid {
    /// Number of week columns.
    pub cols: usize,
    /// Number of day rows (always 7).
    pub rows: usize,
    /// Date of the first cell of the grid (always a Sunday).
    pub start: NaiveDate,
    cells: Vec<Option<crate::github::model::ContributionDay>>,
}

impl Grid {
    /// Builds a grid from a contribution calendar.
    ///
    /// The grid is aligned so that the first column starts on a Sunday, exactly
    /// like GitHub renders it.
    pub fn from_calendar(calendar: &ContributionCalendar) -> Result<Self> {
        if calendar.days.is_empty() {
            return Err(Error::InvalidResponse(
                "the contribution calendar contains no days".to_string(),
            ));
        }

        let first = calendar
            .days
            .iter()
            .map(|day| day.date)
            .min()
            .expect("calendar is not empty");
        let last = calendar
            .days
            .iter()
            .map(|day| day.date)
            .max()
            .expect("calendar is not empty");

        let start = first - Duration::days(days_from_monday(first) as i64);
        let end = last + Duration::days((6 - days_from_monday(last)) as i64);
        let cols = ((end - start).num_days() / 7 + 1) as usize;
        let rows = 7;

        let mut cells = vec![None; cols * rows];
        for day in &calendar.days {
            let col = ((day.date - start).num_days() / 7) as usize;
            let row = days_from_monday(day.date) as usize;
            if col < cols && row < rows {
                cells[row * cols + col] = Some(day.clone());
            }
        }

        Ok(Self {
            cols,
            rows,
            start,
            cells,
        })
    }

    /// Returns the day stored at the given column/row, if any.
    pub fn get(&self, col: usize, row: usize) -> Option<&crate::github::model::ContributionDay> {
        if col >= self.cols || row >= self.rows {
            return None;
        }
        self.cells[row * self.cols + col].as_ref()
    }

    /// Returns `true` when the cell holds at least one contribution.
    pub fn is_food(&self, col: usize, row: usize) -> bool {
        self.get(col, row).is_some_and(|day| day.level > 0)
    }

    /// Every cell that holds at least one contribution.
    pub fn food_cells(&self) -> Vec<(usize, usize)> {
        let mut cells = Vec::new();
        for row in 0..self.rows {
            for col in 0..self.cols {
                if self.is_food(col, row) {
                    cells.push((col, row));
                }
            }
        }
        cells
    }

    /// Total number of contributions in the grid.
    pub fn total(&self) -> i64 {
        self.cells.iter().flatten().map(|day| day.count).sum()
    }
}

/// Number of days between Monday and the given date (Monday is zero).
///
/// GitHub lays the calendar out in weeks that start on Monday, so the first
/// column of the grid holds the remaining days of the first (partial) week.
fn days_from_monday(date: NaiveDate) -> u32 {
    match date.weekday() {
        Weekday::Mon => 0,
        Weekday::Tue => 1,
        Weekday::Wed => 2,
        Weekday::Thu => 3,
        Weekday::Fri => 4,
        Weekday::Sat => 5,
        Weekday::Sun => 6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::github::model::ContributionDay;

    fn day(date: &str, count: i64, level: u8) -> ContributionDay {
        ContributionDay {
            date: NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap(),
            count,
            level,
        }
    }

    #[test]
    fn grid_starts_on_monday() {
        // 2024-01-03 is a Wednesday, so the week starts on 2024-01-01.
        let calendar = ContributionCalendar {
            days: vec![day("2024-01-03", 1, 1)],
            total: 1,
        };
        let grid = Grid::from_calendar(&calendar).unwrap();
        assert_eq!(
            grid.start,
            NaiveDate::parse_from_str("2024-01-01", "%Y-%m-%d").unwrap()
        );
        assert_eq!(grid.rows, 7);
        assert_eq!(grid.cols, 1);
        assert_eq!(grid.get(0, 2).unwrap().date.to_string(), "2024-01-03");
    }

    #[test]
    fn every_day_lands_in_its_own_cell() {
        // A full week plus one day: no day may overwrite another one.
        let days: Vec<ContributionDay> = ["2024-01-01", "2024-01-07", "2024-01-08", "2024-01-14"]
            .iter()
            .map(|date| day(date, 1, 1))
            .collect();
        let calendar = ContributionCalendar { days, total: 4 };
        let grid = Grid::from_calendar(&calendar).unwrap();
        let mut stored = 0;
        for row in 0..grid.rows {
            for col in 0..grid.cols {
                if grid.get(col, row).is_some() {
                    stored += 1;
                }
            }
        }
        assert_eq!(stored, 4);
        assert_eq!(grid.get(0, 6).unwrap().date.to_string(), "2024-01-07");
        assert_eq!(grid.get(1, 0).unwrap().date.to_string(), "2024-01-08");
    }

    #[test]
    fn grid_finds_food() {
        let calendar = ContributionCalendar {
            days: vec![
                day("2024-01-01", 0, 0),
                day("2024-01-02", 5, 2),
                day("2024-01-03", 0, 0),
            ],
            total: 5,
        };
        let grid = Grid::from_calendar(&calendar).unwrap();
        assert_eq!(grid.food_cells(), vec![(0, 1)]);
        assert_eq!(grid.total(), 5);
    }

    #[test]
    fn empty_calendar_is_an_error() {
        assert!(Grid::from_calendar(&ContributionCalendar::default()).is_err());
    }
}
