//! The radar chart of the 3D profile: one axis per kind of activity.

use crate::github::stats::{Axis, ProfileStats};

/// The values of the rings, from the inside out.
const RING_LABELS: [&str; 5] = ["1", "10", "100", "1K", "10K"];

/// The number of rings.
const RINGS: usize = 5;

/// Where the radar chart is drawn.
#[derive(Debug, Clone, Copy)]
pub struct RadarGeometry {
    /// Centre of the chart.
    pub cx: f64,
    /// Centre of the chart.
    pub cy: f64,
    /// Radius of the outer ring.
    pub radius: f64,
}

impl RadarGeometry {
    /// Creates the geometry.
    pub fn new(cx: f64, cy: f64, radius: f64) -> Self {
        Self { cx, cy, radius }
    }

    /// The point of an axis at the given fraction of the radius.
    ///
    /// The first axis points straight up, the others follow clockwise.
    pub fn point(&self, axis_index: usize, fraction: f64) -> (f64, f64) {
        let angle =
            -std::f64::consts::FRAC_PI_2 + axis_index as f64 * 2.0 * std::f64::consts::PI / 5.0;
        (
            self.cx + self.radius * fraction * angle.cos(),
            self.cy + self.radius * fraction * angle.sin(),
        )
    }
}

/// Maps a value to the fraction of the radius, on a logarithmic scale.
pub fn scale(value: u64) -> f64 {
    if value == 0 {
        return 0.0;
    }
    let logarithmic = ((value + 1) as f64).log10() / (10_000.0f64).log10();
    (0.2 + 0.8 * logarithmic).clamp(0.0, 1.0)
}

/// Draws the radar chart and returns the SVG fragment.
pub fn render(
    stats: &ProfileStats,
    geometry: RadarGeometry,
    accent: &str,
    foreground: &str,
    weak: &str,
) -> String {
    let mut svg = String::with_capacity(4 * 1024);

    // rings
    for ring in 1..=RINGS {
        let fraction = ring as f64 / RINGS as f64;
        let mut points = Vec::with_capacity(5);
        for axis in 0..5 {
            let (x, y) = geometry.point(axis, fraction);
            points.push(format!("{x:.1},{y:.1}"));
        }
        svg.push_str(&format!(
            "<polygon points=\"{}\" fill=\"none\" stroke=\"{weak}\" stroke-opacity=\"0.55\" stroke-width=\"1.2\" stroke-dasharray=\"4 4\"/>",
            points.join(" ")
        ));
    }

    // axes
    for axis in 0..5 {
        let (x, y) = geometry.point(axis, 1.0);
        svg.push_str(&format!(
            "<line x1=\"{:.1}\" y1=\"{:.1}\" x2=\"{x:.1}\" y2=\"{y:.1}\" stroke=\"{weak}\" stroke-opacity=\"0.45\" stroke-width=\"1.2\"/>",
            geometry.cx, geometry.cy
        ));
    }

    // the data polygon
    let mut data_points = Vec::with_capacity(5);
    for (index, axis) in Axis::ALL.iter().enumerate() {
        let (x, y) = geometry.point(index, scale(axis.value(stats)));
        data_points.push(format!("{x:.1},{y:.1}"));
    }
    svg.push_str(&format!(
        "<polygon points=\"{}\" fill=\"{accent}\" fill-opacity=\"0.45\" stroke=\"{accent}\" stroke-width=\"3\"/>",
        data_points.join(" ")
    ));

    // axis labels
    for (index, axis) in Axis::ALL.iter().enumerate() {
        let (x, y) = geometry.point(index, 1.18);
        svg.push_str(&format!(
            "<text x=\"{x:.1}\" y=\"{y:.1}\" text-anchor=\"middle\" dominant-baseline=\"middle\" font-size=\"20\" fill=\"{foreground}\">{}</text>",
            axis.label()
        ));
    }

    // ring labels, on the vertical axis
    for (index, label) in RING_LABELS.iter().enumerate() {
        let fraction = (index + 1) as f64 / RINGS as f64;
        let (x, y) = geometry.point(0, fraction);
        svg.push_str(&format!(
            "<text x=\"{x:.1}\" y=\"{y:.1}\" text-anchor=\"middle\" dominant-baseline=\"middle\" font-size=\"13\" fill=\"{weak}\">{label}</text>"
        ));
    }

    svg
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scale_is_logarithmic() {
        assert_eq!(scale(0), 0.0);
        assert!(scale(10) > scale(1));
        assert!(scale(1_000) > scale(100));
        assert!(scale(100_000) <= 1.0);
    }

    #[test]
    fn first_axis_points_up() {
        let geometry = RadarGeometry::new(100.0, 100.0, 50.0);
        let (x, y) = geometry.point(0, 1.0);
        assert!((x - 100.0).abs() < 0.001);
        assert!(y < 100.0);
    }
}
