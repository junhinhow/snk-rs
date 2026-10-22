//! Command line interface.

use std::path::PathBuf;

use clap::Parser;

use crate::error::{Error, Result};
use crate::palette::Palette;
use crate::render::RenderOptions;
use crate::render3d::{Render3DOptions, Style3D};

/// Generate an animated snake that crawls your GitHub contribution graph.
#[derive(Parser, Debug, Clone)]
#[command(
    name = "snk-rs",
    version,
    about = "Generate an animated snake that crawls your GitHub contribution graph",
    long_about = "snk-rs renders the GitHub contribution calendar of a user as an \
                  animated SVG in which a snake eats every day that has contributions.\n\n\
                  Without a token the public contribution page is used, so only public \
                  contributions show up - which is what you want on a profile README. \
                  With a token the GraphQL API is used instead, which also returns the \
                  exact number of contributions per day."
)]
pub struct Cli {
    /// GitHub user whose contribution graph is rendered.
    #[arg(short, long, value_name = "USER")]
    pub user: String,

    /// GitHub token used to read the contribution calendar.
    ///
    /// Can also be provided through the `GITHUB_TOKEN` environment variable.
    #[arg(
        long,
        value_name = "TOKEN",
        env = "GITHUB_TOKEN",
        hide_env_values = true
    )]
    pub token: Option<String>,

    /// File the SVG is written to. Defaults to the standard output.
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<PathBuf>,

    /// Colour palette: `github`, `github-light` or `github-dark`.
    #[arg(short, long, default_value = "github-dark", value_name = "NAME")]
    pub palette: String,

    /// Colour of the snake, as a hex value or a colour name.
    #[arg(long, value_name = "COLOR")]
    pub color_snake: Option<String>,

    /// Colours of the five contribution levels, comma separated.
    #[arg(long, value_name = "COLORS", value_delimiter = ',')]
    pub color_dots: Vec<String>,

    /// Side of a single cell, in pixels.
    #[arg(long, default_value_t = 11, value_name = "PX")]
    pub cell: u32,

    /// Space between two cells, in pixels.
    #[arg(long, default_value_t = 2, value_name = "PX")]
    pub gap: u32,

    /// Space around the grid, in pixels.
    #[arg(long, default_value_t = 4, value_name = "PX")]
    pub margin: u32,

    /// Duration of one animation loop, in seconds.
    #[arg(long, default_value_t = 10.0, value_name = "SECONDS")]
    pub duration: f32,

    /// First day of the period, as `YYYY-MM-DD`.
    #[arg(long, value_name = "DATE")]
    pub from: Option<String>,

    /// Last day of the period, as `YYYY-MM-DD`.
    #[arg(long, value_name = "DATE")]
    pub to: Option<String>,

    /// `User-Agent` header sent to GitHub.
    #[arg(long, value_name = "AGENT")]
    pub user_agent: Option<String>,

    /// What to render: `snake` (the animated snake) or `3d` (the isometric
    /// contribution profile).
    #[arg(long, value_name = "FORMAT", default_value = "snake")]
    pub format: String,

    /// Colour palette of the 3D profile: `green`, `night-green` or
    /// `night-view`.
    #[arg(long, value_name = "STYLE")]
    pub style: Option<String>,

    /// Do not animate the 3D bars.
    #[arg(long)]
    pub no_animate: bool,

    /// Include the contributions of private repositories in the graph.
    ///
    /// Off by default: a profile README should show the public activity.
    #[arg(long)]
    pub include_private: bool,
}

impl Cli {
    /// Parses the `--from` date.
    pub fn from_date(&self) -> Result<Option<chrono::NaiveDate>> {
        parse_date(self.from.as_deref())
    }

    /// Parses the `--to` date.
    pub fn to_date(&self) -> Result<Option<chrono::NaiveDate>> {
        parse_date(self.to.as_deref())
    }

    /// Builds the renderer options out of the command line arguments.
    pub fn render_options(&self) -> Result<RenderOptions> {
        let mut palette = Palette::named(&self.palette)?;
        if let Some(color) = &self.color_snake {
            palette = palette.with_snake(color)?;
        }
        if !self.color_dots.is_empty() {
            palette = palette.with_dots(&self.color_dots)?;
        }

        Ok(RenderOptions {
            palette,
            cell: self.cell,
            gap: self.gap,
            margin: self.margin,
            duration: self.duration,
            title: format!("{}'s GitHub contribution snake", self.user),
        })
    }

    /// Builds the options of the 3D renderer.
    pub fn render3d_options(&self) -> Result<Render3DOptions> {
        let style = match &self.style {
            Some(name) => Style3D::named(name)
                .ok_or_else(|| Error::Config(format!("unknown style `{name}`")))?,
            None => Style3D::night_green(),
        };

        Ok(Render3DOptions {
            style,
            animate: !self.no_animate,
            duration: self.duration.max(0.5),
            title: format!("{}'s GitHub 3D contribution profile", self.user),
            ..Render3DOptions::default()
        })
    }

    /// `true` when the 3D profile should be rendered.
    pub fn is_three_dimensional(&self) -> bool {
        matches!(
            self.format.trim().to_ascii_lowercase().as_str(),
            "3d" | "profile"
        )
    }
}

/// Parses a `YYYY-MM-DD` date.
fn parse_date(value: Option<&str>) -> Result<Option<chrono::NaiveDate>> {
    match value {
        None => Ok(None),
        Some(raw) => chrono::NaiveDate::parse_from_str(raw.trim(), "%Y-%m-%d")
            .map(Some)
            .map_err(|_| Error::InvalidDate(raw.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_is_valid() {
        Cli::command().debug_assert();
    }

    #[test]
    fn parses_arguments() {
        let cli = Cli::try_parse_from([
            "snk-rs",
            "--user",
            "junhinhow",
            "--palette",
            "github",
            "--color-snake",
            "#16A34A",
            "--color-dots",
            "#000000,#111111,#222222,#333333,#444444",
        ])
        .unwrap();
        assert_eq!(cli.user, "junhinhow");
        assert_eq!(cli.palette, "github");
        let options = cli.render_options().unwrap();
        assert_eq!(options.palette.snake, "#16a34a");
        assert_eq!(options.palette.dots[4], "#444444");
    }

    #[test]
    fn rejects_invalid_dates() {
        let cli = Cli::try_parse_from(["snk-rs", "--user", "x", "--from", "01/01/2024"]).unwrap();
        assert!(cli.from_date().is_err());
    }
}
