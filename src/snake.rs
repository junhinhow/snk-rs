//! The snake: how it moves across the grid and which cells it eats.
//!
//! The snake crawls over the *whole* grid in a serpentine (boustrophedon):
//! it sweeps the bottom row, climbs one row and sweeps back, and so on. This
//! fills the board like the classic game and guarantees that every day with
//! contributions is eaten on the way.

use crate::grid::Grid;

/// The planned trajectory of the snake.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SnakePlan {
    /// Every cell the head visits, in order. The first entry is the start cell.
    pub path: Vec<(usize, usize)>,
    /// Cells that hold contributions, together with the step at which the
    /// snake reaches (and eats) them.
    pub eaten: Vec<(usize, usize, usize)>,
}

impl SnakePlan {
    /// Number of movements performed by the head.
    pub fn steps(&self) -> usize {
        self.path.len().saturating_sub(1)
    }
}

/// Plans the whole journey of the snake over the grid.
///
/// The trajectory is a serpentine that starts at the bottom left and ends on
/// the opposite row, visiting every single cell exactly once. The snake eats
/// a cell as soon as it steps onto it, so the days with contributions are
/// naturally collected in the order they are reached.
pub fn plan(grid: &Grid) -> SnakePlan {
    let mut path = Vec::with_capacity(grid.cols * grid.rows);
    let mut eaten = Vec::new();

    let mut row = grid.rows.saturating_sub(1);
    let mut rightwards = true;

    loop {
        let columns: Box<dyn Iterator<Item = usize>> = if rightwards {
            Box::new(0..grid.cols)
        } else {
            Box::new((0..grid.cols).rev())
        };
        for col in columns {
            let step = path.len();
            path.push((col, row));
            if grid.is_food(col, row) {
                eaten.push((col, row, step));
            }
        }
        if row == 0 {
            break;
        }
        row -= 1;
        rightwards = !rightwards;
    }

    SnakePlan { path, eaten }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::github::model::{ContributionCalendar, ContributionDay};
    use chrono::NaiveDate;

    fn calendar(entries: &[(&str, u8)]) -> ContributionCalendar {
        let days = entries
            .iter()
            .map(|(date, level)| ContributionDay {
                date: NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap(),
                count: if *level > 0 { 1 } else { 0 },
                level: *level,
            })
            .collect();
        ContributionCalendar { days, total: 0 }
    }

    #[test]
    fn snake_visits_every_cell_exactly_once() {
        let calendar = calendar(&[("2024-01-03", 2), ("2024-01-10", 3)]);
        let grid = Grid::from_calendar(&calendar).unwrap();
        let plan = plan(&grid);

        assert_eq!(plan.path.len(), grid.cols * grid.rows);
        let mut seen = std::collections::HashSet::new();
        for cell in &plan.path {
            assert!(seen.insert(*cell), "cell {cell:?} visited twice");
        }
    }

    #[test]
    fn snake_eats_every_food_cell() {
        let calendar = calendar(&[
            ("2024-01-01", 0),
            ("2024-01-03", 2),
            ("2024-01-10", 3),
            ("2024-01-15", 1),
        ]);
        let grid = Grid::from_calendar(&calendar).unwrap();
        let plan = plan(&grid);

        let mut eaten: Vec<(usize, usize)> = plan.eaten.iter().map(|(c, r, _)| (*c, *r)).collect();
        eaten.sort_unstable();
        let mut expected = grid.food_cells();
        expected.sort_unstable();
        assert_eq!(eaten, expected);
    }

    #[test]
    fn snake_starts_at_the_bottom_left() {
        let calendar = calendar(&[("2024-01-03", 1)]);
        let grid = Grid::from_calendar(&calendar).unwrap();
        let plan = plan(&grid);
        assert_eq!(plan.path[0], (0, grid.rows - 1));
    }

    #[test]
    fn steps_are_adjacent() {
        let calendar = calendar(&[("2024-01-03", 1), ("2024-01-10", 1), ("2024-02-14", 1)]);
        let grid = Grid::from_calendar(&calendar).unwrap();
        let plan = plan(&grid);
        for window in plan.path.windows(2) {
            let (c0, r0) = window[0];
            let (c1, r1) = window[1];
            let distance = (c1 as isize - c0 as isize).abs() + (r1 as isize - r0 as isize).abs();
            assert_eq!(distance, 1, "snake jumped from {window:?}");
        }
    }

    #[test]
    fn empty_grid_still_fills_the_board() {
        let calendar = calendar(&[("2024-01-03", 0)]);
        let grid = Grid::from_calendar(&calendar).unwrap();
        let plan = plan(&grid);
        assert_eq!(plan.path.len(), grid.cols * grid.rows);
        assert!(plan.eaten.is_empty());
    }
}
