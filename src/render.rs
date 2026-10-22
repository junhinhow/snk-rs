//! The renderer: turns a grid and a snake plan into a self contained SVG.
//!
//! The snake is not drawn as a line but as a wave of cells lighting up in the
//! order the head visits them: the lit cells *are* the body, the newest one is
//! the head. This is what keeps the image readable when the contributions are
//! sparse and the route is a set of long straight lines.

use std::collections::HashSet;

use crate::grid::Grid;
use crate::palette::Palette;
use crate::snake::SnakePlan;

/// How many cells of the body are lit at the same time.
const BODY_CELLS: f32 = 5.0;

/// How the SVG should look.
#[derive(Debug, Clone)]
pub struct RenderOptions {
    /// Colour palette.
    pub palette: Palette,
    /// Side of a single cell, in pixels.
    pub cell: u32,
    /// Space between two cells, in pixels.
    pub gap: u32,
    /// Space around the grid, in pixels.
    pub margin: u32,
    /// Length of one animation loop, in seconds.
    pub duration: f32,
    /// Accessible title of the image.
    pub title: String,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            palette: Palette::github_dark(),
            cell: 11,
            gap: 2,
            margin: 4,
            duration: 10.0,
            title: "GitHub contribution snake".to_string(),
        }
    }
}

/// Renders the animated SVG.
pub fn render(grid: &Grid, plan: &SnakePlan, options: &RenderOptions) -> String {
    let pitch = options.cell + options.gap;
    let width = options.margin * 2 + grid.cols as u32 * pitch - options.gap;
    let height = options.margin * 2 + grid.rows as u32 * pitch - options.gap;
    let duration = if options.duration > 0.0 {
        options.duration
    } else {
        10.0
    };
    let radius = (options.cell / 4).max(1);

    let x_of = |col: usize| options.margin + col as u32 * pitch;
    let y_of = |row: usize| options.margin + row as u32 * pitch;

    let total_steps = plan.path.len().max(1) as f32;
    // How long a single cell stays lit: the body length, in seconds.
    let flash = (BODY_CELLS / total_steps * duration).clamp(0.12, 1.5);
    let eat_flash = (flash * 1.8).min(duration);

    // --- cells -------------------------------------------------------------
    let mut cells = String::new();
    for row in 0..grid.rows {
        for col in 0..grid.cols {
            if let Some(day) = grid.get(col, row) {
                let level = day.level.min(4);
                cells.push_str(&format!(
                    "<rect class=\"l{level}\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"{radius}\"/>",
                    x_of(col),
                    y_of(row),
                    options.cell,
                    options.cell
                ));
            }
        }
    }

    // --- the snake: cells that light up in turn ----------------------------
    let food: HashSet<(usize, usize)> = plan.eaten.iter().map(|(c, r, _)| (*c, *r)).collect();
    let mut segments = String::new();
    for (step, (col, row)) in plan.path.iter().enumerate() {
        let delay = (step as f32 / total_steps) * duration;
        let class = if food.contains(&(*col, *row)) {
            "eat"
        } else {
            "seg"
        };
        segments.push_str(&format!(
            "<rect class=\"{class}\" style=\"animation-delay:{delay:.3}s\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"{radius}\"/>",
            x_of(*col),
            y_of(*row),
            options.cell,
            options.cell
        ));
    }

    // --- assembly ----------------------------------------------------------
    let palette = &options.palette;
    let mut svg = String::with_capacity(32 * 1024);

    svg.push_str(&format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width}\" height=\"{height}\" viewBox=\"0 0 {width} {height}\" role=\"img\" aria-label=\"{}\">",
        escape(&options.title)
    ));
    svg.push_str(&format!(
        "<title>{}</title><desc>{}</desc>",
        escape(&options.title),
        escape(&format!(
            "Animated snake over a GitHub contribution grid with {} contributions.",
            grid.total()
        ))
    ));

    svg.push_str("<style>");
    svg.push_str(&format!(
        ":root{{--cp:{};--ce:{};--cs:{};--c0:{};--c1:{};--c2:{};--c3:{};--c4:{}}}",
        palette.page,
        palette.background,
        palette.snake,
        palette.dots[0],
        palette.dots[1],
        palette.dots[2],
        palette.dots[3],
        palette.dots[4]
    ));
    svg.push_str(".bg{fill:var(--cp)}");
    svg.push_str(".l0{fill:var(--c0)}.l1{fill:var(--c1)}.l2{fill:var(--c2)}.l3{fill:var(--c3)}.l4{fill:var(--c4)}");
    // Every segment animates over the whole loop, but only flashes inside the
    // short window when the head passes over it. The delay of each cell places
    // that window at the right moment.
    let window = flash / duration * 100.0;
    let eat_window = eat_flash / duration * 100.0;
    svg.push_str(".seg{fill:var(--cs);opacity:0;animation:seg ");
    svg.push_str(&format!("{duration:.3}s linear infinite}}"));
    svg.push_str(&format!(
        ".eat{{fill:var(--c4);opacity:0;animation:eat {duration:.3}s linear infinite}}"
    ));
    svg.push_str(&format!(
        "@keyframes seg{{0%{{opacity:0}}{:.3}%{{opacity:1}}{:.3}%{{opacity:1}}{:.3}%{{opacity:0}}100%{{opacity:0}}}}",
        window * 0.18,
        window * 0.55,
        window
    ));
    svg.push_str(&format!(
        "@keyframes eat{{0%{{opacity:0}}{:.3}%{{opacity:1}}{:.3}%{{opacity:1}}{:.3}%{{opacity:0}}100%{{opacity:0}}}}",
        eat_window * 0.18,
        eat_window * 0.55,
        eat_window
    ));
    svg.push_str("</style>");

    svg.push_str(&format!(
        "<rect class=\"bg\" width=\"{width}\" height=\"{height}\" rx=\"{radius}\"/>"
    ));
    svg.push_str(&format!("<g class=\"grid\">{cells}</g>"));
    svg.push_str(&format!("<g class=\"snake\">{segments}</g>"));
    svg.push_str("</svg>");

    svg
}

/// Escapes the characters that are not allowed in XML text nodes.
fn escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::github::model::{ContributionCalendar, ContributionDay};
    use chrono::NaiveDate;

    fn grid_with_food() -> Grid {
        let days = vec![
            ContributionDay {
                date: NaiveDate::parse_from_str("2024-01-03", "%Y-%m-%d").unwrap(),
                count: 3,
                level: 2,
            },
            ContributionDay {
                date: NaiveDate::parse_from_str("2024-01-10", "%Y-%m-%d").unwrap(),
                count: 8,
                level: 3,
            },
        ];
        Grid::from_calendar(&ContributionCalendar { days, total: 11 }).unwrap()
    }

    #[test]
    fn renders_expected_structure() {
        let grid = grid_with_food();
        let plan = crate::snake::plan(&grid);
        let svg = render(&grid, &plan, &RenderOptions::default());

        assert!(svg.starts_with("<svg "));
        assert!(svg.ends_with("</svg>"));
        assert!(svg.contains("<style>"));
        assert!(svg.contains("@keyframes seg"));
        assert!(svg.contains("class=\"l2\""));
        assert!(svg.contains("class=\"l3\""));
        assert!(svg.contains("class=\"seg\""));
        assert!(svg.contains("class=\"eat\""));
        assert!(svg.contains("class=\"snake\""));
    }

    #[test]
    fn one_segment_per_visited_cell() {
        let grid = grid_with_food();
        let plan = crate::snake::plan(&grid);
        let svg = render(&grid, &plan, &RenderOptions::default());

        let segments = svg.matches("class=\"seg\"").count();
        let eats = svg.matches("class=\"eat\"").count();
        assert_eq!(segments + eats, plan.path.len());
        assert_eq!(eats, plan.eaten.len());
    }

    #[test]
    fn the_grid_is_visible_against_the_page() {
        let palette = Palette::github_dark();
        assert_ne!(palette.page, palette.background);
        let light = Palette::github_light();
        assert_ne!(light.page, light.background);
    }

    #[test]
    fn escapes_xml() {
        assert_eq!(escape("a&b<c>\"d\""), "a&amp;b&lt;c&gt;&quot;d&quot;");
    }
}
