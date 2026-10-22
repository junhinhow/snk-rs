//! The renderer: turns a grid and a snake plan into a self contained SVG.
//!
//! The snake is a solid, rounded stroke that crawls along the serpentine route
//! using a moving dash, with a head (and eyes) following the leading edge.
//! Days with contributions flash brighter as the snake eats them.

use crate::grid::Grid;
use crate::palette::Palette;
use crate::snake::SnakePlan;

/// How many cells long the snake body is.
const BODY_CELLS: f32 = 10.0;

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

    // Centre of a cell, used to route the snake.
    let cx_of = |col: usize| options.margin + col as u32 * pitch + options.cell / 2;
    let cy_of = |row: usize| options.margin + row as u32 * pitch + options.cell / 2;

    let total_steps = plan.steps().max(1) as f32;
    let path_length = total_steps * pitch as f32;
    let body_length = (BODY_CELLS * pitch as f32).min(path_length);

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

    // --- the route (through every cell centre) ----------------------------
    let mut route = String::new();
    for (index, (col, row)) in plan.path.iter().enumerate() {
        let command = if index == 0 { "M" } else { "L" };
        route.push_str(&format!("{command} {} {}", cx_of(*col), cy_of(*row)));
    }

    // --- cells eaten by the snake -----------------------------------------
    // A day flashes bright green for a short moment as the head passes over it.
    let mut eaten = String::new();
    for (col, row, step) in &plan.eaten {
        let arrival = (*step as f32 / total_steps) * duration;
        eaten.push_str(&format!(
            "<rect class=\"eat\" style=\"animation-delay:{arrival:.3}s\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"{radius}\"/>",
            options.margin + *col as u32 * pitch,
            options.margin + *row as u32 * pitch,
            options.cell,
            options.cell
        ));
    }

    // --- assembly ----------------------------------------------------------
    let palette = &options.palette;
    let mut svg = String::with_capacity(16 * 1024);

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
    // The body: a dash of `body_length` that slides from the start of the
    // route to its end over one loop.
    svg.push_str(&format!(
        ".body{{fill:none;stroke:var(--cs);stroke-width:{};stroke-linecap:round;stroke-linejoin:round;stroke-dasharray:{} {};animation:crawl {duration:.3}s linear infinite}}",
        options.cell,
        fmt(body_length),
        fmt(path_length)
    ));
    svg.push_str(&format!(
        "@keyframes crawl{{from{{stroke-dashoffset:0}}to{{stroke-dashoffset:{}}}}}",
        fmt(-(path_length - body_length))
    ));
    // The flash of a day being eaten: it lights up only inside the short
    // window when the head is on top of it, then fades for the rest of the loop.
    let window = (BODY_CELLS / total_steps * 100.0).clamp(1.0, 10.0);
    svg.push_str(".eat{fill:var(--c4);opacity:0;animation:eat ");
    svg.push_str(&format!("{duration:.3}s linear infinite}}"));
    svg.push_str(&format!(
        "@keyframes eat{{0%{{opacity:0}}{:.3}%{{opacity:1}}{:.3}%{{opacity:1}}{:.3}%{{opacity:0}}100%{{opacity:0}}}}",
        window * 0.2,
        window * 0.6,
        window
    ));
    svg.push_str("</style>");

    svg.push_str(&format!(
        "<rect class=\"bg\" width=\"{width}\" height=\"{height}\" rx=\"{radius}\"/>"
    ));
    svg.push_str(&format!("<g class=\"grid\">{cells}</g>"));

    if plan.path.len() > 1 {
        svg.push_str(&format!(
            "<path id=\"route\" d=\"{route}\" fill=\"none\" stroke=\"none\"/>"
        ));
        svg.push_str(&format!("<path class=\"body\" d=\"{route}\"/>"));
        // Head at the leading edge, with two eyes.
        let lead = body_length / path_length;
        svg.push_str(&format!(
            "<g><circle r=\"{:.1}\" fill=\"var(--cs)\"/><circle cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"var(--ce)\"/><circle cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"var(--ce)\"/><animateMotion dur=\"{duration:.3}s\" repeatCount=\"indefinite\" keyPoints=\"{lead:.4};1\" keyTimes=\"0;1\"><mpath href=\"#route\"/></animateMotion></g>",
            options.cell as f32 / 2.0,
            fmt(-(options.cell as f32 / 4.0)),
            fmt(-(options.cell as f32 / 4.0)),
            fmt((options.cell as f32 / 6.0).max(1.0)),
            fmt(options.cell as f32 / 4.0),
            fmt(-(options.cell as f32 / 4.0)),
            fmt((options.cell as f32 / 6.0).max(1.0)),
        ));
    }
    svg.push_str(&format!("<g class=\"eaten\">{eaten}</g>"));
    svg.push_str("</svg>");

    svg
}

/// Formats an `f32` without trailing zeros.
fn fmt(value: f32) -> String {
    if value.fract().abs() < 1e-4 {
        format!("{value:.0}")
    } else {
        format!("{value:.2}")
    }
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
        assert!(svg.contains("@keyframes crawl"));
        assert!(svg.contains("class=\"l2\""));
        assert!(svg.contains("class=\"body\""));
        assert!(svg.contains("animateMotion"));
        assert!(svg.contains("class=\"eat\""));
    }

    #[test]
    fn route_visits_every_cell() {
        let grid = grid_with_food();
        let plan = crate::snake::plan(&grid);
        let svg = render(&grid, &plan, &RenderOptions::default());

        let commands = svg.matches('L').count() + svg.matches('M').count();
        assert!(commands >= plan.path.len(), "route missing cells");
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
