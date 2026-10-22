//! The snake: how it moves across the grid and which cells it eats.

use std::collections::{HashSet, VecDeque};

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
/// The snake always heads for the closest cell that still holds contributions
/// and eats it on arrival. The search is a breadth first search over the grid,
/// so the snake never gets stuck behind its own trail.
pub fn plan(grid: &Grid) -> SnakePlan {
    let mut food: HashSet<(usize, usize)> = grid.food_cells().into_iter().collect();
    let start = start_cell(grid);

    let mut path = vec![start];
    let mut eaten = Vec::new();
    if food.remove(&start) {
        eaten.push((start.0, start.1, 0));
    }

    // Safety net: never plan more movements than there are cells in the grid.
    let max_steps = grid.cols * grid.rows * 4;

    while let Some(next) = next_step(grid, *path.last().expect("path is never empty"), &food) {
        let step = path.len();
        path.push(next);
        if food.remove(&next) {
            eaten.push((next.0, next.1, step));
        }
        if path.len() > max_steps {
            break;
        }
    }

    SnakePlan { path, eaten }
}

/// The cell the snake starts from: bottom left, like every good snake.
fn start_cell(grid: &Grid) -> (usize, usize) {
    let row = grid.rows.saturating_sub(1);
    (0, row)
}

/// Finds the next cell to move to: one step along the shortest path to the
/// closest cell that still holds contributions.
fn next_step(
    grid: &Grid,
    head: (usize, usize),
    food: &HashSet<(usize, usize)>,
) -> Option<(usize, usize)> {
    if food.is_empty() {
        return None;
    }

    let mut previous: Vec<Option<(usize, usize)>> = vec![None; grid.cols * grid.rows];
    let mut visited: Vec<bool> = vec![false; grid.cols * grid.rows];
    let mut queue: VecDeque<(usize, usize)> = VecDeque::new();

    queue.push_back(head);
    visited[head.1 * grid.cols + head.0] = true;

    let mut target = None;
    while let Some(current) = queue.pop_front() {
        if current != head && food.contains(&current) {
            target = Some(current);
            break;
        }
        for neighbour in neighbours(grid, current) {
            let index = neighbour.1 * grid.cols + neighbour.0;
            if !visited[index] {
                visited[index] = true;
                previous[index] = Some(current);
                queue.push_back(neighbour);
            }
        }
    }

    let target = target?;
    // Walk the path backwards to the head and return the cell next to it.
    let mut cursor = target;
    loop {
        let parent = previous[cursor.1 * grid.cols + cursor.0]?;
        if parent == head {
            return Some(cursor);
        }
        cursor = parent;
    }
}

/// The four orthogonal neighbours of a cell that are inside the grid.
fn neighbours(grid: &Grid, (col, row): (usize, usize)) -> Vec<(usize, usize)> {
    let mut result = Vec::with_capacity(4);
    if col > 0 {
        result.push((col - 1, row));
    }
    if col + 1 < grid.cols {
        result.push((col + 1, row));
    }
    if row > 0 {
        result.push((col, row - 1));
    }
    if row + 1 < grid.rows {
        result.push((col, row + 1));
    }
    result
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
    fn empty_grid_produces_a_single_cell_path() {
        let calendar = calendar(&[("2024-01-03", 0)]);
        let grid = Grid::from_calendar(&calendar).unwrap();
        let plan = plan(&grid);
        assert_eq!(plan.path.len(), 1);
        assert!(plan.eaten.is_empty());
    }
}
