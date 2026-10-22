//! Colour palettes for the rendered contribution graph.

use crate::error::{Error, Result};

/// A colour palette: background, the five contribution levels and the snake.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Palette {
    /// Human readable name of the palette.
    pub name: String,
    /// Colour of the canvas drawn behind the grid.
    pub page: String,
    /// Colour of a day without contributions.
    pub background: String,
    /// Colours for contribution levels `0..=4`.
    pub dots: [String; 5],
    /// Colour of the snake itself.
    pub snake: String,
    /// Colour of the snake's head (brighter than the body).
    pub head: String,
}

impl Palette {
    /// The light palette used by GitHub itself.
    pub fn github_light() -> Self {
        Self {
            name: "github-light".to_string(),
            page: "#ffffff".to_string(),
            background: "#ebedf0".to_string(),
            dots: [
                "#ebedf0".to_string(),
                "#9be9a8".to_string(),
                "#40c463".to_string(),
                "#30a14e".to_string(),
                "#216e39".to_string(),
            ],
            snake: "#8957e5".to_string(),
            head: "#a371f7".to_string(),
        }
    }

    /// The dark palette used by GitHub itself.
    pub fn github_dark() -> Self {
        Self {
            name: "github-dark".to_string(),
            page: "#0d1117".to_string(),
            background: "#161b22".to_string(),
            dots: [
                "#161b22".to_string(),
                "#0e4429".to_string(),
                "#006d32".to_string(),
                "#26a641".to_string(),
                "#39d353".to_string(),
            ],
            snake: "#8957e5".to_string(),
            head: "#a371f7".to_string(),
        }
    }

    /// Looks a palette up by name.
    ///
    /// Supported names: `github`, `github-light`, `github-dark` and `dark`.
    pub fn named(name: &str) -> Result<Self> {
        match name.trim().to_ascii_lowercase().as_str() {
            "github" | "github-light" | "light" => Ok(Self::github_light()),
            "github-dark" | "dark" => Ok(Self::github_dark()),
            other => Err(Error::Config(format!(
                "unknown palette `{other}` (expected github, github-light or github-dark)"
            ))),
        }
    }

    /// Overrides the snake colour.
    ///
    /// The head is derived from it so the two always belong to the same hue
    /// (a purple head on a green body would look broken).
    pub fn with_snake(mut self, color: &str) -> Result<Self> {
        self.snake = normalize_color(color)?;
        self.head = brighten(&self.snake, 1.5);
        Ok(self)
    }

    /// Overrides the five contribution level colours.
    pub fn with_dots(mut self, colors: &[String]) -> Result<Self> {
        if colors.len() != 5 {
            return Err(Error::Config(format!(
                "expected 5 dot colours (level 0..=4), got {}",
                colors.len()
            )));
        }
        let mut dots: [String; 5] = std::array::from_fn(|_| String::new());
        for (slot, color) in dots.iter_mut().zip(colors) {
            *slot = normalize_color(color)?;
        }
        self.dots = dots;
        Ok(self)
    }
}

/// Normalises a colour to the `#rrggbb` notation.
///
/// Accepts `#rgb`, `#rrggbb`, `rgb`/`rrggbb` and a small set of names.
pub fn normalize_color(input: &str) -> Result<String> {
    let raw = input.trim();
    let lower = raw.to_ascii_lowercase();

    if let Some(hex) = lower.strip_prefix('#') {
        return expand_hex(hex);
    }
    if let Some((_, hex)) = NAMED_COLORS.iter().find(|(name, _)| *name == lower) {
        return Ok((*hex).to_string());
    }
    if (lower.len() == 3 || lower.len() == 6) && lower.chars().all(|c| c.is_ascii_hexdigit()) {
        return expand_hex(&lower);
    }

    Err(Error::InvalidColor(input.to_string()))
}

/// Expands `rgb` into `rrggbb` and validates the digits.
fn expand_hex(hex: &str) -> Result<String> {
    let expanded = match hex.len() {
        3 => hex.chars().map(|c| format!("{c}{c}")).collect::<String>(),
        6 => hex.to_string(),
        _ => return Err(Error::InvalidColor(hex.to_string())),
    };
    if !expanded.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(Error::InvalidColor(hex.to_string()));
    }
    Ok(format!("#{expanded}"))
}

/// Lightens a `#rrggbb` colour by `ratio`, capping every channel at 255.
///
/// Used to derive a head colour that reads as the same hue as the body but
/// clearly brighter.
fn brighten(hex: &str, ratio: f32) -> String {
    let channel =
        |at: usize| u8::from_str_radix(hex.get(at..at + 2).unwrap_or("00"), 16).unwrap_or(0);
    let lift = |c: u8| ((c as f32 * ratio).round() as i64).clamp(0, 255) as u8;
    format!(
        "#{:02x}{:02x}{:02x}",
        lift(channel(1)),
        lift(channel(3)),
        lift(channel(5))
    )
}

/// A few colour names, so `--color-snake green` just works.
const NAMED_COLORS: &[(&str, &str)] = &[
    ("black", "#000000"),
    ("white", "#ffffff"),
    ("gray", "#808080"),
    ("grey", "#808080"),
    ("green", "#00c853"),
    ("lime", "#00e676"),
    ("emerald", "#16a34a"),
    ("teal", "#14b8a6"),
    ("blue", "#2563eb"),
    ("sky", "#38bdf8"),
    ("purple", "#8957e5"),
    ("violet", "#8b5cf6"),
    ("magenta", "#d946ef"),
    ("pink", "#ec4899"),
    ("red", "#ef4444"),
    ("orange", "#f97316"),
    ("amber", "#f59e0b"),
    ("yellow", "#eab308"),
    ("cyan", "#06b6d4"),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expands_short_hex() {
        assert_eq!(normalize_color("#abc").unwrap(), "#aabbcc");
        assert_eq!(normalize_color("abc").unwrap(), "#aabbcc");
    }

    #[test]
    fn keeps_long_hex() {
        assert_eq!(normalize_color("#16A34A").unwrap(), "#16a34a");
    }

    #[test]
    fn resolves_names() {
        assert_eq!(normalize_color("green").unwrap(), "#00c853");
        assert_eq!(normalize_color("Emerald").unwrap(), "#16a34a");
    }

    #[test]
    fn rejects_garbage() {
        assert!(normalize_color("#12345").is_err());
        assert!(normalize_color("not-a-color").is_err());
    }

    #[test]
    fn palettes_have_five_dots() {
        assert_eq!(Palette::github_dark().dots.len(), 5);
        assert_eq!(Palette::named("github").unwrap().dots.len(), 5);
        assert!(Palette::named("nope").is_err());
    }

    #[test]
    fn dot_override_requires_five_colors() {
        let palette = Palette::github_dark();
        assert!(palette.clone().with_dots(&[]).is_err());
        let five = vec![
            "#000000".to_string(),
            "#111111".to_string(),
            "#222222".to_string(),
            "#333333".to_string(),
            "#444444".to_string(),
        ];
        assert!(palette.with_dots(&five).is_ok());
    }
}
