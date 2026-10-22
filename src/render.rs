//! The renderer: turns a grid and a snake plan into a self contained SVG.

use crate::grid::Grid;
use crate::palette::Palette;
use crate::snake::SnakePlan;

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

    let total_steps = plan.steps().max(1);
    let path_length = total_steps as f32 * pitch as f32;

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

    // --- cells eaten by the snake -----------------------------------------
    const COVER_DURATION: f32 = 0.15;
    let mut covers = String::new();
    for (col, row, step) in &plan.eaten {
        let arrival = (*step as f32 / total_steps as f32) * duration;
        let delay = (arrival - COVER_DURATION).max(0.0);
        covers.push_str(&format!(
            "<rect class=\"cover\" style=\"animation-delay:{delay:.3}s\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"{radius}\"/>",
            x_of(*col),
            y_of(*row),
            options.cell,
            options.cell
        ));
    }

    // --- trajectory --------------------------------------------------------
    let corners = corners(&plan.path);
    let mut route = String::new();
    for (index, (col, row, _)) in corners.iter().enumerate() {
        if index == 0 {
            route.push_str(&format!("M {} {}", x_of(*col), y_of(*row)));
        } else {
            route.push_str(&format!(" L {} {}", x_of(*col), y_of(*row)));
        }
    }

    // --- head keyframes ----------------------------------------------------
    let mut keyframes = String::new();
    for (col, row, step) in &corners {
        let percent = (*step as f32 / total_steps as f32) * 100.0;
        keyframes.push_str(&format!(
            "{percent:.3}%{{transform:translate({}px,{}px)}}",
            x_of(*col),
            y_of(*row)
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
        ":root{{--ce:{};--cs:{};--c0:{};--c1:{};--c2:{};--c3:{};--c4:{}}}",
        palette.background,
        palette.snake,
        palette.dots[0],
        palette.dots[1],
        palette.dots[2],
        palette.dots[3],
        palette.dots[4]
    ));
    svg.push_str(".bg{fill:var(--ce)}");
    svg.push_str(".l0{fill:var(--c0)}.l1{fill:var(--c1)}.l2{fill:var(--c2)}.l3{fill:var(--c3)}.l4{fill:var(--c4)}");
    svg.push_str(&format!(
        ".cover{{fill:var(--ce);opacity:0;animation:cover {COVER_DURATION}s linear forwards}}"
    ));
    svg.push_str(&format!(
        ".trail{{fill:none;stroke:var(--cs);stroke-width:{};stroke-linecap:round;stroke-linejoin:round;stroke-dasharray:{path_length};animation:drawtrail {duration}s linear infinite}}",
        (options.cell as f32 * 0.55).max(2.0)
    ));
    svg.push_str(&format!(
        ".head{{fill:var(--cs);stroke:var(--ce);stroke-width:1;animation:headmove {duration}s linear infinite}}"
    ));
    svg.push_str("@keyframes cover{from{opacity:0}to{opacity:1}}");
    svg.push_str(&format!(
        "@keyframes drawtrail{{from{{stroke-dashoffset:{path_length}}}to{{stroke-dashoffset:0}}}}"
    ));
    svg.push_str(&format!("@keyframes headmove{{{keyframes}}}"));
    svg.push_str("</style>");

    svg.push_str(&format!(
        "<rect class=\"bg\" width=\"{width}\" height=\"{height}\" rx=\"{radius}\"/>"
    ));
    svg.push_str(&format!("<g class=\"grid\">{cells}</g>"));
    if !covers.is_empty() {
        svg.push_str(&format!("<g class=\"covers\">{covers}</g>"));
    }
    if corners.len() > 1 {
        svg.push_str(&format!("<path class=\"trail\" d=\"{route}\"/>"));
    }
    svg.push_str(&format!(
        "<rect class=\"head\" x=\"0\" y=\"0\" width=\"{}\" height=\"{}\" rx=\"{radius}\"/>",
        (options.cell as f32 * 0.8).round().max(4.0) as u32,
        (options.cell as f32 * 0.8).round().max(4.0) as u32
    ));
    svg.push_str("</svg>");

    svg
}

/// Reduces a path to its corner points, keeping the step index of each corner.
///
/// Collinear points are dropped: they do not change the shape of the line and
/// they would only make the animation bigger.
fn corners(path: &[(usize, usize)]) -> Vec<(usize, usize, usize)> {
    if path.is_empty() {
        return Vec::new();
    }
    if path.len() < 3 {
        return path
            .iter()
            .enumerate()
            .map(|(index, (col, row))| (*col, *row, index))
            .collect();
    }

    let mut result = vec![(path[0].0, path[0].1, 0usize)];
    for index in 1..path.len() - 1 {
        let previous = path[index - 1];
        let current = path[index];
        let next = path[index + 1];
        let direction_in = (
            current.0 as isize - previous.0 as isize,
            current.1 as isize - previous.1 as isize,
        );
        let direction_out = (
            next.0 as isize - current.0 as isize,
            next.1 as isize - current.1 as isize,
        );
        if direction_in != direction_out {
            result.push((current.0, current.1, index));
        }
    }
    let last = path.len() - 1;
    result.push((path[last].0, path[last].1, last));
    result
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
        assert!(svg.contains("@keyframes headmove"));
        assert!(svg.contains("class=\"l2\""));
        assert!(svg.contains("class=\"l3\""));
        assert!(svg.contains("class=\"cover\""));
        assert!(svg.contains("class=\"trail\""));
        assert!(svg.contains("class=\"head\""));
    }

    #[test]
    fn corners_drop_collinear_points() {
        let path = vec![(0, 0), (1, 0), (2, 0), (3, 0), (3, 1)];
        let corners = corners(&path);
        assert_eq!(corners, vec![(0, 0, 0), (3, 0, 3), (3, 1, 4)]);
    }

    #[test]
    fn escapes_xml() {
        assert_eq!(escape("a&b<c>\"d\""), "a&amp;b&lt;c&gt;&quot;d&quot;");
    }
}
