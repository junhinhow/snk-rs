//! The 3D contribution profile: isometric bars, a radar chart, a language
//! donut and the counters at the bottom.
//!
//! The idea comes from the great `yoshi389111/github-profile-3d-contrib`
//! action; this is an independent implementation with the same layout.

pub mod donut;
pub mod iso;
pub mod radar;

use crate::github::stats::ProfileStats;
use crate::grid::Grid;
use crate::render3d::donut::DonutGeometry;
use crate::render3d::iso::{bar_height, points, project, Faces};
use crate::render3d::radar::RadarGeometry;

/// Width of the canvas.
pub const WIDTH: f64 = 1280.0;
/// Height of the canvas.
pub const HEIGHT: f64 = 850.0;

/// A colour palette of the 3D profile.
#[derive(Debug, Clone, Copy)]
pub struct Style3D {
    /// Name of the palette.
    pub name: &'static str,
    /// Background colour.
    pub background: &'static str,
    /// Colour of the regular text.
    pub foreground: &'static str,
    /// Colour of the highlighted numbers.
    pub strong: &'static str,
    /// Colour of the secondary text.
    pub weak: &'static str,
    /// Colour of the radar chart.
    pub accent: &'static str,
    /// Colour of a day without contributions.
    pub empty: &'static str,
    /// Colours of the five levels: `[level][top, left, right]`.
    pub levels: [[&'static str; 3]; 5],
}

impl Style3D {
    /// The light palette.
    pub fn green() -> Self {
        Self {
            name: "green",
            background: "#ffffff",
            foreground: "#0b0b16",
            strong: "#111133",
            weak: "#7a7a7a",
            accent: "#47a042",
            empty: "#d5d5d5",
            levels: [
                ["#ebedf0", "#dfe2e6", "#d2d5da"],
                ["#9be9a8", "#86d495", "#71bd80"],
                ["#40c463", "#37a957", "#2f8f4c"],
                ["#30a14e", "#298c44", "#22763b"],
                ["#216e39", "#1d5e32", "#194f2b"],
            ],
        }
    }

    /// The dark palette with green blocks.
    pub fn night_green() -> Self {
        Self {
            name: "night-green",
            background: "#0d1117",
            foreground: "#c9d1d9",
            strong: "#e6edf3",
            weak: "#6e7681",
            accent: "#47a042",
            empty: "#2b3138",
            levels: [
                ["#2b3138", "#242a30", "#1e2329"],
                ["#0e4429", "#0c3a23", "#0a301d"],
                ["#006d32", "#00592a", "#004723"],
                ["#26a641", "#208c37", "#1a732d"],
                ["#39d353", "#30ac46", "#288f3a"],
            ],
        }
    }

    /// The dark palette with blue blocks.
    pub fn night_view() -> Self {
        Self {
            name: "night-view",
            background: "#0d1117",
            foreground: "#c9d1d9",
            strong: "#e6edf3",
            weak: "#6e7681",
            accent: "#58a6ff",
            empty: "#21262d",
            levels: [
                ["#21262d", "#1c2126", "#171b1f"],
                ["#0e2f4d", "#0c2740", "#0a2034"],
                ["#11558c", "#0e4876", "#0b3b61"],
                ["#1f6feb", "#1a5fce", "#1552b0"],
                ["#58a6ff", "#4a94e6", "#3d7fcc"],
            ],
        }
    }

    /// Looks a palette up by name.
    pub fn named(name: &str) -> Option<Self> {
        match name.trim().to_ascii_lowercase().as_str() {
            "green" | "light" => Some(Self::green()),
            "night-green" | "night_green" | "dark" => Some(Self::night_green()),
            "night-view" | "night_view" | "blue" => Some(Self::night_view()),
            _ => None,
        }
    }
}

/// How the 3D profile should look.
#[derive(Debug, Clone)]
pub struct Render3DOptions {
    /// Colour palette.
    pub style: Style3D,
    /// Side of a cell in the grid space.
    pub cell: f64,
    /// Maximum height of a bar, in pixels.
    pub max_height: f64,
    /// Whether the bars grow when the image is opened.
    pub animate: bool,
    /// Length of the growth animation, in seconds.
    pub duration: f32,
    /// Accessible title of the image.
    pub title: String,
}

impl Default for Render3DOptions {
    fn default() -> Self {
        Self {
            style: Style3D::night_green(),
            cell: 15.0,
            max_height: 160.0,
            animate: true,
            duration: 3.0,
            title: "GitHub 3D contribution profile".to_string(),
        }
    }
}

/// Renders the whole 3D profile.
pub fn render3d(grid: &Grid, stats: &ProfileStats, options: &Render3DOptions) -> String {
    let style = &options.style;

    // biggest day of the period, used to scale the bars
    let max_count = grid
        .get_all()
        .iter()
        .map(|day| day.count)
        .max()
        .unwrap_or(0);

    // place the grid so the band starts close to the top left corner
    let (offset_x, offset_y) = grid_offset(grid, options);

    let mut svg = String::with_capacity(512 * 1024);

    svg.push_str(&format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{WIDTH}\" height=\"{HEIGHT}\" viewBox=\"0 0 {WIDTH} {HEIGHT}\" role=\"img\" aria-label=\"{}\">",
        escape(&options.title)
    ));
    svg.push_str(&format!(
        "<title>{}</title><desc>{}</desc>",
        escape(&options.title),
        escape(&format!(
            "3D view of the GitHub contribution graph with {} contributions.",
            stats.total_contributions
        ))
    ));
    svg.push_str("<style>");
    svg.push_str(&format!(
        ":root{{--bg:{};--fg:{};--strong:{};--weak:{};--accent:{};--empty:{}}}",
        style.background, style.foreground, style.strong, style.weak, style.accent, style.empty
    ));
    svg.push_str("text{font-family:'Ubuntu','Helvetica','Arial',sans-serif}");
    svg.push_str("</style>");
    svg.push_str(&format!(
        "<rect x=\"0\" y=\"0\" width=\"{WIDTH}\" height=\"{HEIGHT}\" class=\"fill-bg\" fill=\"{}\"/>",
        style.background
    ));

    svg.push_str(&format!(
        "<g class=\"contrib\" transform=\"translate({offset_x:.1},{offset_y:.1})\">"
    ));
    svg.push_str(&render_bars(grid, options, max_count));
    svg.push_str("</g>");

    // radar chart on the top right
    let radar = RadarGeometry::new(WIDTH - 40.0 - 200.0, 70.0 + 200.0, 200.0);
    svg.push_str(&radar::render(
        stats,
        radar,
        style.accent,
        style.foreground,
        style.weak,
    ));

    // language donut on the bottom left
    let donut = DonutGeometry::new(40.0 + 130.0, HEIGHT - 70.0 - 130.0, 130.0);
    svg.push_str(&donut::render(stats, donut, style.foreground));

    // counters at the bottom
    svg.push_str(&render_counters(stats, style));

    svg.push_str("</svg>");
    svg
}

/// Draws every bar of the grid.
fn render_bars(grid: &Grid, options: &Render3DOptions, max_count: i64) -> String {
    let style = &options.style;
    let mut svg = String::with_capacity(256 * 1024);
    let total_cells = (grid.cols * grid.rows).max(1) as f64;

    for col in 0..grid.cols {
        for row in 0..grid.rows {
            let Some(day) = grid.get(col, row) else {
                continue;
            };
            let level = (day.level.min(4)) as usize;
            let height = bar_height(day.count, max_count, options.max_height);
            let faces = Faces::build(col as f64, row as f64, options.cell, height);
            let flat = Faces::flat(col as f64, row as f64, options.cell);

            let colors = &style.levels[level];
            let delay = if options.animate {
                let position = (col * grid.rows + row) as f64 / total_cells;
                format!("{:.3}s", position * options.duration as f64 * 0.55)
            } else {
                String::new()
            };

            for (index, (face, color)) in [
                (&faces.top, colors[0]),
                (&faces.left, colors[1]),
                (&faces.right, colors[2]),
            ]
            .iter()
            .enumerate()
            {
                svg.push_str(&format!(
                    "<polygon points=\"{}\" fill=\"{color}\">",
                    points(face)
                ));
                if options.animate {
                    let from = match index {
                        0 => points(&flat.top),
                        1 => points(&flat.left),
                        _ => points(&flat.right),
                    };
                    let to = points(face);
                    svg.push_str(&format!(
                        "<animate attributeName=\"points\" from=\"{from}\" to=\"{to}\" begin=\"{delay}\" dur=\"{:.3}s\" fill=\"freeze\" calcMode=\"spline\" keyTimes=\"0;1\" keySplines=\"0.2 0 0.8 1\"/>",
                        options.duration as f64 * 0.45
                    ));
                }
                svg.push_str("</polygon>");
            }
        }
    }

    svg
}

/// The counters printed at the bottom of the image.
fn render_counters(stats: &ProfileStats, style: &Style3D) -> String {
    let mut svg = String::with_capacity(2 * 1024);
    let baseline = HEIGHT - 40.0;

    let contributions_x = 400.0;
    svg.push_str(&format!(
        "<text x=\"{contributions_x:.0}\" y=\"{baseline:.0}\" text-anchor=\"end\" font-size=\"32\" font-weight=\"bold\" fill=\"{}\">{}</text>",
        style.strong,
        thousands(stats.total_contributions.max(0))
    ));
    svg.push_str(&format!(
        "<text x=\"{:.0}\" y=\"{baseline:.0}\" font-size=\"24\" fill=\"{}\">contributions</text>",
        contributions_x + 12.0,
        style.foreground
    ));

    let stars_x = 620.0;
    svg.push_str(&format!(
        "<g transform=\"translate({:.0} {:.0}) scale(1.7)\" fill=\"{}\">",
        stars_x - 22.0,
        baseline - 24.0,
        style.foreground
    ));
    svg.push_str(STAR_PATH);
    svg.push_str("</g>");
    svg.push_str(&format!(
        "<text x=\"{stars_x:.0}\" y=\"{baseline:.0}\" font-size=\"26\" fill=\"{}\">{}</text>",
        style.foreground,
        thousands(stats.stars as i64)
    ));

    let pr_x = 730.0;
    svg.push_str(&format!(
        "<g transform=\"translate({:.0} {:.0}) scale(1.7)\" fill=\"{}\">",
        pr_x - 22.0,
        baseline - 24.0,
        style.foreground
    ));
    svg.push_str(PULL_REQUEST_PATH);
    svg.push_str("</g>");
    svg.push_str(&format!(
        "<text x=\"{pr_x:.0}\" y=\"{baseline:.0}\" font-size=\"26\" fill=\"{}\">{}</text>",
        style.foreground,
        thousands(stats.pull_requests as i64)
    ));

    // period
    svg.push_str(&format!(
        "<text x=\"{:.0}\" y=\"60\" text-anchor=\"end\" font-size=\"18\" fill=\"{}\">{}</text>",
        WIDTH - 40.0,
        style.weak,
        escape(&stats.period)
    ));

    svg
}

/// Finds the translation that places the band of bars nicely.
fn grid_offset(grid: &Grid, options: &Render3DOptions) -> (f64, f64) {
    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    for col in 0..=grid.cols {
        for row in 0..=grid.rows {
            let point = project(
                col as f64 * options.cell,
                row as f64 * options.cell,
                options.max_height,
            );
            min_x = min_x.min(point.x);
            min_y = min_y.min(point.y);
        }
    }
    (140.0 - min_x, 120.0 - min_y)
}

/// Formats a number with thousand separators.
fn thousands(value: i64) -> String {
    let text = value.abs().to_string();
    let mut out = String::new();
    for (index, character) in text.chars().enumerate() {
        if index > 0 && (text.len() - index) % 3 == 0 {
            out.push(',');
        }
        out.push(character);
    }
    if value < 0 {
        format!("-{out}")
    } else {
        out
    }
}

/// Escapes the characters that are not allowed in XML text nodes.
pub fn escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

const STAR_PATH: &str = "<path fill-rule=\"evenodd\" d=\"M8 .25a.75.75 0 01.673.418l1.882 3.815 4.21.612a.75.75 0 01.416 1.279l-3.046 2.97.719 4.196a.75.75 0 01-1.088.791L12 18.347l-3.766 1.984a.75.75 0 01-1.088-.79l.719-4.197-3.046-2.97a.75.75 0 01.416-1.28l4.21-.611L7.327.668A.75.75 0 018 .25z\"/>";

const PULL_REQUEST_PATH: &str = "<path fill-rule=\"evenodd\" d=\"M7.177 3.073L9.573.677A.75.75 0 0110 .854v4.792a.75.75 0 01-.427.668L7.177 7.427a.75.75 0 01-.627-.174A.75.75 0 016 6.793V3.75A.75.75 0 017.177 3.073zM10 1.73v6.926a2.25 2.25 0 101.5 0V3.5a2.25 2.25 0 10-1.5 0zM3.75 2.5a.75.75 0 100 1.5.75.75 0 000-1.5zm-2.25.75a.75.75 0 110 1.5.75.75 0 001.5 0zM11 2.5h-1V4h1a2.25 2.25 0 110 4.5h-2.25a.75.75 0 010-1.5H10A.75.75 0 0010 6.5v8.879l1.22-1.22a.75.75 0 111.06 1.06l-2.5 2.5a.75.75 0 01-1.06 0l-2.5-2.5a.75.75 0 011.06-1.06l1.22 1.22V6.5A.75.75 0 0111 5.75h-1V4h1A2.25 2.25 0 1013.5 2.25h-2.5a.75.75 0 010-1.5h2.5z\"/>";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thousands_are_grouped() {
        assert_eq!(thousands(0), "0");
        assert_eq!(thousands(999), "999");
        assert_eq!(thousands(1000), "1,000");
        assert_eq!(thousands(215671), "215,671");
    }

    #[test]
    fn styles_are_resolved() {
        assert!(Style3D::named("night-green").is_some());
        assert!(Style3D::named("nope").is_none());
    }

    #[test]
    fn escape_protects_xml() {
        assert_eq!(escape("a&b<c>"), "a&amp;b&lt;c&gt;");
    }
}
