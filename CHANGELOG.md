# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- The snake crosses the grid at half speed: one loop now lasts 20s instead of
  10s. The body length (9 cells) and the grid stay exactly the same.
- `--duration` now only drives the snake; the 3D bars have their own
  `--3d-duration` flag (10s) so the two animations stay independent.
- The snake crawls over the whole grid in a serpentine instead of only hopping
  between the days with contributions, so the animation fills the board like
  the classic game.
- The body is drawn as grid cells that light up in turn -- the leading cell is
  the head in a brighter shade -- with a head and days that flash as they are
  eaten. This is plain CSS animation with no SMIL, so the image renders
  identically in every browser.

### Fixed

- The grid was invisible on the dark palette: the background rectangle used the
  same colour as the empty cells. The canvas now has its own colour (`page`)
  and the empty cells are drawn against it.
## [0.2.0] - 2026-10-22

### Added

- The 3D contribution profile (`--format 3d`): isometric bars with three faces
  per day animated with SMIL, a radar chart of commits, issues, pull requests,
  reviews and repositories on a logarithmic scale, a language donut and the
  counters at the bottom.
- Palettes `green`, `night-green` and `night-view` for the 3D profile.
- `--style` and `--no-animate` options.
- `--include-private`: the calendar stays public by default, the token is only
  used for the extra statistics.
- Extra statistics through the GraphQL API (`github::stats::ProfileStats`).

## [0.1.0] - 2026-10-22

### Added

- Initial release.
- Animated SVG renderer of the GitHub contribution graph, with a snake that
  eats every day holding contributions.
- Two data sources: the public contribution page (no token required) and the
  GraphQL API (exact counts, optional private contributions).
- Palettes `github`, `github-light` and `github-dark`, plus custom snake and
  cell colours.
- Library API (`snk_rs`) next to the `snk-rs` binary.
- GitHub Actions and Forgejo Actions workflows.

[Unreleased]: https://github.com/junhinhow/snk-rs/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/junhinhow/snk-rs/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/junhinhow/snk-rs/releases/tag/v0.1.0
