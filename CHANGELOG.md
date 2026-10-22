# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[Unreleased]: https://github.com/junhinhow/snk-rs/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/junhinhow/snk-rs/releases/tag/v0.1.0
