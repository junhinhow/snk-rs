//! # snk-rs
//!
//! An animated snake that crawls your GitHub contribution graph, rendered as a
//! self-contained SVG.
//!
//! The idea and the visual language come from the wonderful
//! [`Platane/snk`](https://github.com/Platane/snk) project (MIT licensed).
//! This is an independent Rust implementation written from scratch, with a
//! different architecture: a small, dependency-light library that can be
//! embedded in other tools, a public-data-first fetching strategy and a
//! renderer that produces a compact animated SVG.
//!
//! ## Quick start
//!
//! ```no_run
//! use snk_rs::{render, RenderOptions};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let calendar = snk_rs::github::GitHubClient::new("junhinhow", None)
//!     .fetch_calendar()?;
//! let grid = snk_rs::grid::Grid::from_calendar(&calendar)?;
//! let plan = snk_rs::snake::plan(&grid);
//! let svg = render(&grid, &plan, &RenderOptions::default());
//! print!("{svg}");
//! # Ok(())
//! # }
//! ```

pub mod cli;
pub mod error;
pub mod github;
pub mod grid;
pub mod palette;
pub mod render;
pub mod render3d;
pub mod snake;

pub use error::{Error, Result};
pub use grid::Grid;
pub use palette::Palette;
pub use render::{render, RenderOptions};
pub use render3d::{render3d, Render3DOptions, Style3D};
pub use snake::{plan, SnakePlan};

/// Version of the crate, useful when embedding the renderer.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
