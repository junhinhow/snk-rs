//! Error types shared across the crate.

use thiserror::Error;

/// Convenient alias for results produced by this crate.
pub type Result<T> = std::result::Result<T, Error>;

/// Every failure mode that can happen while fetching or rendering.
#[derive(Debug, Error)]
pub enum Error {
    /// The HTTP request to GitHub failed.
    #[error("http request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// GitHub answered, but the payload did not look like a contribution calendar.
    #[error("unexpected response from GitHub: {0}")]
    InvalidResponse(String),

    /// The contribution calendar could not be parsed.
    #[error("failed to parse contribution calendar: {0}")]
    Parse(String),

    /// The requested user does not exist or has no reachable calendar.
    #[error("user not found or calendar unavailable: {0}")]
    NotFound(String),

    /// A date was not in the `YYYY-MM-DD` format.
    #[error("invalid date `{0}`, expected YYYY-MM-DD")]
    InvalidDate(String),

    /// A colour was neither a hex value nor a known colour name.
    #[error("invalid colour `{0}`")]
    InvalidColor(String),

    /// Filesystem failure while writing the output.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// The renderer was asked to draw something impossible.
    #[error("invalid configuration: {0}")]
    Config(String),
}
