//! The language donut of the 3D profile.

use crate::github::stats::ProfileStats;

/// Where the donut is drawn.
#[derive(Debug, Clone, Copy)]
pub struct DonutGeometry {
    /// Centre of the donut.
    pub cx: f64,
    /// Centre of the donut.
    pub cy: f64,
    /// Outer radius.
    pub outer: f64,
    /// Inner radius.
    pub inner: f64,
}

impl DonutGeometry {
    /// Creates the geometry.
    pub fn new(cx: f64, cy: f64, outer: f64) -> Self {
        Self {
            cx,
            cy,
            outer,
            inner: outer * 0.58,
        }
    }
}

/// Builds the path of a donut segment.
pub fn segment(cx: f64, cy: f64, outer: f64, inner: f64, start: f64, end: f64) -> String {
    let large = if end - start > std::f64::consts::PI {
        1
    } else {
        0
    };
    let (x1, y1) = (cx + outer * start.cos(), cy + outer * start.sin());
    let (x2, y2) = (cx + outer * end.cos(), cy + outer * end.sin());
    let (x3, y3) = (cx + inner * end.cos(), cy + inner * end.sin());
    let (x4, y4) = (cx + inner * start.cos(), cy + inner * start.sin());
    format!(
        "M {x1:.1} {y1:.1} A {outer:.1} {outer:.1} 0 {large} 1 {x2:.1} {y2:.1} \
         L {x3:.1} {y3:.1} A {inner:.1} {inner:.1} 0 {large} 0 {x4:.1} {y4:.1} Z"
    )
}

/// Draws the language donut and its legend.
pub fn render(stats: &ProfileStats, geometry: DonutGeometry, foreground: &str) -> String {
    let mut svg = String::with_capacity(2 * 1024);

    let total: u64 = stats.languages.iter().map(|language| language.size).sum();
    if total == 0 {
        return svg;
    }

    let mut angle = -std::f64::consts::FRAC_PI_2;
    for language in &stats.languages {
        let share = language.size as f64 / total as f64;
        let sweep = share * std::f64::consts::TAU;
        svg.push_str(&format!(
            "<path d=\"{}\" fill=\"{}\"/>",
            segment(
                geometry.cx,
                geometry.cy,
                geometry.outer,
                geometry.inner,
                angle,
                angle + sweep
            ),
            language.color
        ));
        angle += sweep;
    }

    // legend
    let legend_x = geometry.cx + geometry.outer + 30.0;
    let mut legend_y = geometry.cy - geometry.outer + 6.0;
    for language in stats.languages.iter().take(8) {
        let share = language.size as f64 / total as f64 * 100.0;
        svg.push_str(&format!(
            "<rect x=\"{legend_x:.1}\" y=\"{:.1}\" width=\"16\" height=\"16\" fill=\"{}\"/>",
            legend_y - 12.0,
            language.color
        ));
        svg.push_str(&format!(
            "<text x=\"{:.1}\" y=\"{:.1}\" font-size=\"19\" fill=\"{foreground}\">{}</text>",
            legend_x + 26.0,
            legend_y,
            crate::render3d::escape(&language.name)
        ));
        svg.push_str(&format!(
            "<text x=\"{:.1}\" y=\"{:.1}\" font-size=\"19\" text-anchor=\"end\" fill=\"{foreground}\">{share:.1}%</text>",
            legend_x + 240.0,
            legend_y
        ));
        legend_y += 30.0;
    }

    svg
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segment_is_a_closed_path() {
        let path = segment(0.0, 0.0, 100.0, 60.0, 0.0, 1.0);
        assert!(path.starts_with("M "));
        assert!(path.ends_with("Z"));
        assert!(path.contains("A 100.0 100.0"));
    }

    #[test]
    fn full_circle_uses_the_large_arc_flag() {
        let path = segment(0.0, 0.0, 100.0, 60.0, 0.0, std::f64::consts::TAU);
        assert!(path.contains(" 1 "));
    }
}
