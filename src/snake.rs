//! The snake: how it moves across the grid and which cells it eats.
//!
//! The snake hunts the contribution cells instead of sweeping the board: it
//! works through the grid one intensity level at a time (level 1 first, then
//! 2, 3 and 4) and, for each level, walks to the nearest cell it can still
//! reach. Empty cells and cells of the current level or lower are walkable,
//! while brighter cells act as walls until their turn comes -- this is what
//! makes the snake wind across the calendar instead of running along a row.

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

/// The starting cell: bottom left, like every good snake.
fn start_cell(grid: &Grid) -> (usize, usize) {
    (0, grid.rows.saturating_sub(1))
}

/// Plans the whole journey of the snake over the grid.
///
/// The snake starts at the bottom left and repeatedly walks to the nearest
/// contribution cell it can still reach. It may cross empty cells and cells of
/// the current intensity level or lower; brighter cells are walls. When no
/// food of the current level is reachable the wall limit is raised by one, so
/// the snake keeps hunting until every contribution has been eaten.
pub fn plan(grid: &Grid) -> SnakePlan {
    let start = start_cell(grid);

    let mut path = vec![start];
    let mut eaten = Vec::new();
    let mut eaten_set: HashSet<(usize, usize)> = HashSet::new();

    if grid.is_food(start.0, start.1) {
        eaten.push((start.0, start.1, 0));
        eaten_set.insert(start);
    }

    // Brightest level the head is allowed to walk through right now.
    let walkable = 1u8;

    while let Some(segment) = hunt(
        grid,
        *path.last().expect("path is never empty"),
        &eaten_set,
        walkable,
    ) {
        for cell in segment {
            let step = path.len();
            path.push(cell);
            if grid.is_food(cell.0, cell.1) && eaten_set.insert(cell) {
                eaten.push((cell.0, cell.1, step));
            }
        }
    }

    SnakePlan { path, eaten }
}

/// Raises the wall limit until some food becomes reachable, then walks to it.
///
/// Returns the cells of the walk (excluding the current head) or `None` when
/// nothing is left to eat.
fn hunt(
    grid: &Grid,
    head: (usize, usize),
    eaten: &HashSet<(usize, usize)>,
    mut walkable: u8,
) -> Option<Vec<(usize, usize)>> {
    loop {
        if let Some(segment) = bfs_to_food(grid, head, eaten, walkable) {
            return Some(segment);
        }
        if walkable >= 4 {
            return None;
        }
        walkable += 1;
    }
}

/// Breadth first search from the head to the nearest reachable, uneaten cell.
///
/// The walkable set is every empty cell plus every cell whose level does not
/// exceed `walkable`; the target is the first uneaten contribution cell of
/// that set found by the search, which is also the closest one.
fn bfs_to_food(
    grid: &Grid,
    head: (usize, usize),
    eaten: &HashSet<(usize, usize)>,
    walkable: u8,
) -> Option<Vec<(usize, usize)>> {
    let cols = grid.cols;
    let index = |(col, row): (usize, usize)| row * cols + col;

    let mut previous: Vec<Option<(usize, usize)>> = vec![None; cols * grid.rows];
    let mut visited = vec![false; cols * grid.rows];
    let mut queue: VecDeque<(usize, usize)> = VecDeque::new();

    visited[index(head)] = true;
    queue.push_back(head);

    let mut target = None;
    while let Some(current) = queue.pop_front() {
        if current != head && !eaten.contains(&current) && level_of(grid, current) >= 1 {
            target = Some(current);
            break;
        }
        for neighbour in neighbours(grid, current) {
            let idx = index(neighbour);
            if !visited[idx] && is_walkable(grid, neighbour, eaten, walkable) {
                visited[idx] = true;
                previous[idx] = Some(current);
                queue.push_back(neighbour);
            }
        }
    }

    let target = target?;
    // Walk the chain of parents back to the head and reverse it.
    let mut segment = Vec::new();
    let mut cursor = target;
    while cursor != head {
        segment.push(cursor);
        cursor = previous[index(cursor)]?;
    }
    segment.reverse();
    Some(segment)
}

/// Whether the head may step onto a cell.
fn is_walkable(
    grid: &Grid,
    cell: (usize, usize),
    eaten: &HashSet<(usize, usize)>,
    walkable: u8,
) -> bool {
    if eaten.contains(&cell) {
        return true;
    }
    level_of(grid, cell) <= walkable
}

/// The intensity level of a cell, or `0` when the day has no contribution.
fn level_of(grid: &Grid, (col, row): (usize, usize)) -> u8 {
    grid.get(col, row).map(|day| day.level).unwrap_or(0)
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
    fn the_snake_hunts_instead_of_sweeping_the_board() {
        // A wide grid (ten weeks) whose only contributions sit in the very
        // first column. The snake should walk straight to them and stop, not
        // fill the empty columns to its right.
        let start = NaiveDate::parse_from_str("2024-01-01", "%Y-%m-%d").unwrap();
        let food = NaiveDate::parse_from_str("2024-01-03", "%Y-%m-%d").unwrap();
        let days: Vec<ContributionDay> = (0..70)
            .map(|offset| {
                let date = start + chrono::Duration::days(offset);
                let is_food = date == food;
                ContributionDay {
                    date,
                    count: i64::from(is_food),
                    level: u8::from(is_food),
                }
            })
            .collect();
        let grid = Grid::from_calendar(&ContributionCalendar { days, total: 1 }).unwrap();
        let plan = plan(&grid);

        assert!(
            grid.cols >= 9,
            "the grid is too narrow to tell hunting from sweeping: {} columns",
            grid.cols
        );
        assert!(
            plan.path.len() < grid.cols * grid.rows / 2,
            "the snake filled the board ({} of {} cells) instead of hunting",
            plan.path.len(),
            grid.cols * grid.rows
        );
    }

    #[test]
    fn brighter_food_waits_for_its_turn() {
        // A level 4 cell boxed in by empty space is only reachable once the
        // wall limit reaches 4; the snake must still eat it in the end.
        let calendar = calendar(&[("2024-01-03", 4), ("2024-01-20", 1)]);
        let grid = Grid::from_calendar(&calendar).unwrap();
        let plan = plan(&grid);
        assert_eq!(plan.eaten.len(), 2);
    }

    #[test]
    fn empty_grid_yields_a_single_cell_path() {
        let calendar = calendar(&[("2024-01-03", 0)]);
        let grid = Grid::from_calendar(&calendar).unwrap();
        let plan = plan(&grid);
        assert_eq!(plan.path.len(), 1);
        assert!(plan.eaten.is_empty());
    }
}
