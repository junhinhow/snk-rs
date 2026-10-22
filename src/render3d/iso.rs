//! Isometric projection used by the 3D contribution graph.

/// Cosine of 30 degrees, the horizontal factor of the projection.
pub const COS_30: f64 = 0.866_025_403_784_438_6;
/// Sine of 30 degrees, the vertical factor of the projection.
pub const SIN_30: f64 = 0.5;

/// A point on the canvas.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    /// Horizontal position.
    pub x: f64,
    /// Vertical position.
    pub y: f64,
}

impl Point {
    /// Creates a point.
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// The `x,y` pair used by the `points` attribute of an SVG polygon.
    pub fn as_pair(self) -> String {
        format!("{:.1},{:.1}", self.x, self.y)
    }
}

/// Projects a point of the grid space into the canvas.
///
/// The grid lives in the `x`/`y` plane and grows towards the viewer in `z`.
pub fn project(x: f64, y: f64, z: f64) -> Point {
    Point {
        x: (x - y) * COS_30,
        y: (x + y) * SIN_30 - z,
    }
}

/// The three visible faces of a bar.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Faces {
    /// The top of the bar.
    pub top: [Point; 4],
    /// The face pointing to the lower left.
    pub left: [Point; 4],
    /// The face pointing to the lower right.
    pub right: [Point; 4],
}

impl Faces {
    /// The faces of a bar with no height, used as the start of the animation.
    pub fn flat(col: f64, row: f64, size: f64) -> Self {
        Self::build(col, row, size, 0.0)
    }

    /// Builds the faces of a bar.
    pub fn build(col: f64, row: f64, size: f64, height: f64) -> Self {
        let x = col * size;
        let y = row * size;

        let a = project(x, y, height);
        let b = project(x + size, y, height);
        let c = project(x + size, y + size, height);
        let d = project(x, y + size, height);

        let b0 = project(x + size, y, 0.0);
        let c0 = project(x + size, y + size, 0.0);
        let d0 = project(x, y + size, 0.0);

        Self {
            top: [a, b, c, d],
            left: [d, c, c0, d0],
            right: [b, c, c0, b0],
        }
    }
}

/// Formats the four points of a face.
pub fn points(face: &[Point; 4]) -> String {
    face.iter()
        .map(|point| point.as_pair())
        .collect::<Vec<_>>()
        .join(" ")
}

/// The scale of the bars: the height in pixels of a day with `count`
/// contributions, given the biggest day of the period.
pub fn bar_height(count: i64, max_count: i64, max_height: f64) -> f64 {
    if count <= 0 || max_count <= 0 {
        return 0.0;
    }
    let ratio = (1.0 + count as f64).ln() / (1.0 + max_count as f64).ln();
    (ratio * max_height).clamp(1.0, max_height)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn projection_moves_right_and_down_with_x() {
        let origin = project(0.0, 0.0, 0.0);
        let right = project(10.0, 0.0, 0.0);
        assert!(right.x > origin.x);
        assert!(right.y > origin.y);
    }

    #[test]
    fn projection_moves_left_and_down_with_y() {
        let origin = project(0.0, 0.0, 0.0);
        let down = project(0.0, 10.0, 0.0);
        assert!(down.x < origin.x);
        assert!(down.y > origin.y);
    }

    #[test]
    fn height_moves_the_bar_up() {
        let flat = project(0.0, 0.0, 0.0);
        let tall = project(0.0, 0.0, 50.0);
        assert!(tall.y < flat.y);
        assert!((tall.x - flat.x).abs() < f64::EPSILON);
    }

    #[test]
    fn heights_grow_with_the_count() {
        let small = bar_height(1, 100, 120.0);
        let big = bar_height(50, 100, 120.0);
        assert!(big > small);
        assert!(small > 0.0);
        assert_eq!(bar_height(0, 100, 120.0), 0.0);
    }

    #[test]
    fn flat_bar_has_no_visible_side() {
        let faces = Faces::flat(0.0, 0.0, 10.0);
        assert_eq!(faces.left[0], faces.left[3]);
        assert_eq!(faces.right[0], faces.right[3]);
    }
}
