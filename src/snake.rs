//! The snake: how it moves across the grid and which cells it eats.
//!
//! The snake hunts the contribution cells instead of sweeping the board: it
//! works through the grid one intensity level at a time (level 1 first, then
//! 2, 3 and 4) and, for each level, walks to the nearest cell it can reach.
//! Empty cells and cells of the current level or lower are walkable, brighter
//! cells are walls until their turn comes. Eaten food cells are avoided when
//! possible so the snake winds around them instead of walking back through
//! the cells it just passed; when there is no other route it still passes
//! through, so every contribution gets eaten.

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

/// Plans the whole journey of the snake over the grid.
pub fn plan(grid: &Grid) -> SnakePlan {
    let start = start_cell(grid);

    let mut path = vec![start];
    let mut eaten = Vec::new();
    let mut eaten_set: HashSet<(usize, usize)> = HashSet::new();

    if grid.is_food(start.0, start.1) {
        eaten.push((start.0, start.1, 0));
        eaten_set.insert(start);
    }

    let mut walkable = 1u8;

    loop {
        // Is there any uneaten food left?
        let remaining: Vec<(usize, usize)> = (0..grid.rows)
            .flat_map(|r| (0..grid.cols).map(move |c| (c, r)))
            .filter(|&(c, r)| grid.is_food(c, r) && !eaten_set.contains(&(c, r)))
            .collect();
        if remaining.is_empty() {
            break;
        }

        let head = *path.last().expect("path is never empty");

        // BFS to the nearest uneaten food of level <= walkable.
        // First try: avoid already-eaten food cells (winding path).
        let target = bfs_to_food(grid, head, &eaten_set, walkable, true);
        let target = match target {
            Some(t) => t,
            None => {
                // Fallback: allow walking through eaten cells.
                match bfs_to_food(grid, head, &eaten_set, walkable, false) {
                    Some(t) => t,
                    None => {
                        if walkable < 4 {
                            walkable += 1;
                            continue;
                        }
                        break;
                    }
                }
            }
        };

        // Take the full BFS path to the target.
        let segment = bfs_path(grid, head, target, &eaten_set, walkable, true);
        let segment = match segment {
            Some(s) => s,
            None => {
                // Fallback: allow walking through eaten cells.
                match bfs_path(grid, head, target, &eaten_set, walkable, false) {
                    Some(s) => s,
                    None => {
                        if walkable < 4 {
                            walkable += 1;
                            continue;
                        }
                        break;
                    }
                }
            }
        };

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

/// BFS from the head to find the nearest uneaten food cell of level <= walkable.
fn bfs_to_food(
    grid: &Grid,
    head: (usize, usize),
    eaten: &HashSet<(usize, usize)>,
    walkable: u8,
    avoid_eaten: bool,
) -> Option<(usize, usize)> {
    let cols = grid.cols;
    let index = |(col, row): (usize, usize)| row * cols + col;

    let mut visited = vec![false; cols * grid.rows];
    let mut queue: VecDeque<(usize, usize)> = VecDeque::new();

    visited[index(head)] = true;
    queue.push_back(head);

    while let Some(current) = queue.pop_front() {
        if current != head && !eaten.contains(&current) {
            let lvl = level_of(grid, current);
            if lvl >= 1 && lvl <= walkable {
                return Some(current);
            }
        }
        for neighbour in neighbours(grid, current) {
            let idx = index(neighbour);
            if visited[idx] {
                continue;
            }
            let passable = if eaten.contains(&neighbour) {
                true
            } else {
                level_of(grid, neighbour) <= walkable
            };
            if !passable {
                continue;
            }
            if avoid_eaten && eaten.contains(&neighbour) {
                continue;
            }
            visited[idx] = true;
            queue.push_back(neighbour);
        }
    }

    None
}

/// BFS path from the head to the target. Returns the cells of the walk
/// (excluding the current head).
fn bfs_path(
    grid: &Grid,
    head: (usize, usize),
    target: (usize, usize),
    eaten: &HashSet<(usize, usize)>,
    walkable: u8,
    avoid_eaten: bool,
) -> Option<Vec<(usize, usize)>> {
    if head == target {
        return Some(Vec::new());
    }

    let cols = grid.cols;
    let index = |(col, row): (usize, usize)| row * cols + col;

    let mut previous: Vec<Option<(usize, usize)>> = vec![None; cols * grid.rows];
    let mut visited = vec![false; cols * grid.rows];
    let mut queue: VecDeque<(usize, usize)> = VecDeque::new();

    visited[index(head)] = true;
    queue.push_back(head);

    while let Some(current) = queue.pop_front() {
        if current == target {
            let mut segment = Vec::new();
            let mut cursor = target;
            while cursor != head {
                segment.push(cursor);
                cursor = previous[index(cursor)]?;
            }
            segment.reverse();
            return Some(segment);
        }
        for neighbour in neighbours(grid, current) {
            let idx = index(neighbour);
            if visited[idx] {
                continue;
            }
            let passable = if eaten.contains(&neighbour) || neighbour == target {
                true
            } else {
                level_of(grid, neighbour) <= walkable
            };
            if !passable {
                continue;
            }
            if avoid_eaten && eaten.contains(&neighbour) {
                continue;
            }
            visited[idx] = true;
            previous[idx] = Some(current);
            queue.push_back(neighbour);
        }
    }

    None
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
