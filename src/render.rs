//! The renderer: turns a grid and a snake plan into a self contained SVG.
//!
//! The snake is drawn as grid cells that light up in turn: the body is a run
//! of cells holding the snake colour, the leading cell is the head in a
//! brighter shade, and days with contributions flash as the head eats them.
//! Everything is plain CSS animation, so the image renders identically in
//! every browser and in the GitHub image proxy.

use crate::grid::Grid;
use crate::palette::Palette;
use crate::snake::SnakePlan;

/// How many cells long the snake body is.
const BODY_CELLS: usize = 9;

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
            cell: 12,
            gap: 4,
            margin: 4,
            duration: 20.0,
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

    let steps = plan.path.len().max(1) as f32;
    // A cell is lit for as long as the head sits on it or just behind it.
    let body_window = (BODY_CELLS as f32 / steps) * 100.0;
    let head_window = (1.0 / steps) * 100.0;
    let food_window = head_window;

    // --- cells -------------------------------------------------------------
    let mut cells = String::new();
    for row in 0..grid.rows {
        for col in 0..grid.cols {
            if let Some(day) = grid.get(col, row) {
                let level = day.level.min(4);
                cells.push_str(&format!(
                    "<rect class=\"l{level}\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"{radius}\"/>",
                    options.margin + col as u32 * pitch,
                    options.margin + row as u32 * pitch,
                    options.cell,
                    options.cell
                ));
            }
        }
    }

    // --- the snake ---------------------------------------------------------
    let mut snake = String::new();
    for (step, (col, row)) in plan.path.iter().enumerate() {
        let x = options.margin + *col as u32 * pitch;
        let y = options.margin + *row as u32 * pitch;
        let delay = (step as f32 / steps) * duration;
        // Body.
        snake.push_str(&format!(
            "<rect class=\"seg\" style=\"animation-delay:{delay:.3}s\" x=\"{x}\" y=\"{y}\" width=\"{}\" height=\"{}\" rx=\"{radius}\"/>",
            options.cell, options.cell
        ));
        // Head, drawn on top of the body at the same moment.
        snake.push_str(&format!(
            "<rect class=\"head\" style=\"animation-delay:{delay:.3}s\" x=\"{x}\" y=\"{y}\" width=\"{}\" height=\"{}\" rx=\"{radius}\"/>",
            options.cell, options.cell
        ));
    }

    // --- days eaten by the snake ------------------------------------------
    let mut eaten = String::new();
    for (col, row, step) in &plan.eaten {
        let delay = (*step as f32 / steps) * duration;
        eaten.push_str(&format!(
            "<rect class=\"eat\" style=\"animation-delay:{delay:.3}s\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"{radius}\"/>",
            options.margin + *col as u32 * pitch,
            options.margin + *row as u32 * pitch,
            options.cell,
            options.cell
        ));
    }

    // --- assembly ----------------------------------------------------------
    let palette = &options.palette;
    let mut svg = String::with_capacity(48 * 1024);

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
        ":root{{--cp:{};--ce:{};--cs:{};--ch:{};--c0:{};--c1:{};--c2:{};--c3:{};--c4:{}}}",
        palette.page,
        palette.background,
        palette.snake,
        palette.head,
        palette.dots[0],
        palette.dots[1],
        palette.dots[2],
        palette.dots[3],
        palette.dots[4]
    ));
    svg.push_str(".bg{fill:var(--cp)}");
    svg.push_str(".l0{fill:var(--c0)}.l1{fill:var(--c1)}.l2{fill:var(--c2)}.l3{fill:var(--c3)}.l4{fill:var(--c4)}");
    // Each of the three roles is lit only inside its own window of the loop,
    // then stays hidden for the rest of it.
    svg.push_str(&format!(
        ".seg{{fill:var(--cs);opacity:0;animation:seg {duration:.3}s linear infinite}}"
    ));
    svg.push_str(&format!(
        ".head{{fill:var(--ch);opacity:0;animation:head {duration:.3}s linear infinite}}"
    ));
    svg.push_str(&format!(
        ".eat{{fill:var(--c4);opacity:0;animation:eat {duration:.3}s linear infinite}}"
    ));
    svg.push_str(&keyframes("seg", body_window));
    svg.push_str(&keyframes("head", head_window));
    svg.push_str(&keyframes("eat", food_window));
    svg.push_str("</style>");

    svg.push_str(&format!(
        "<rect class=\"bg\" width=\"{width}\" height=\"{height}\" rx=\"{radius}\"/>"
    ));
    svg.push_str(&format!("<g class=\"grid\">{cells}</g>"));
    svg.push_str(&format!("<g class=\"snake\">{snake}</g>"));
    svg.push_str(&format!("<g class=\"eaten\">{eaten}</g>"));
    svg.push_str("</svg>");

    svg
}

/// Builds the keyframes that light a cell for `window` percent of the loop.
///
/// The edges are made as sharp as the browser allows so the snake looks like
/// a solid object stepping from cell to cell rather than a fading glow.
fn keyframes(name: &str, window: f32) -> String {
    let window = window.clamp(0.05, 95.0);
    let a = window * 0.2;
    let b = window * 0.6;
    format!(
        "@keyframes {name}{{0%{{opacity:0}}{a:.3}%{{opacity:1}}{b:.3}%{{opacity:1}}{window:.3}%{{opacity:0}}100%{{opacity:0}}}}"
    )
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
        assert!(svg.contains("@keyframes head"));
        assert!(svg.contains("class=\"l2\""));
        assert!(svg.contains("class=\"seg\""));
        assert!(svg.contains("class=\"head\""));
        assert!(svg.contains("class=\"eat\""));
        // no smil, no dash: everything is css
        assert!(!svg.contains("animateMotion"));
        assert!(!svg.contains("stroke-dasharray"));
    }

    #[test]
    fn one_body_and_one_head_per_visited_cell() {
        let grid = grid_with_food();
        let plan = crate::snake::plan(&grid);
        let svg = render(&grid, &plan, &RenderOptions::default());

        assert_eq!(svg.matches("class=\"seg\"").count(), plan.path.len());
        assert_eq!(svg.matches("class=\"head\"").count(), plan.path.len());
        assert_eq!(svg.matches("class=\"eat\"").count(), plan.eaten.len());
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
