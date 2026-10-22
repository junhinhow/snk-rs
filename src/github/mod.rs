//! Fetching the contribution calendar from GitHub.

pub mod graphql;
pub mod html;
pub mod model;
pub mod stats;

use chrono::NaiveDate;
use reqwest::blocking::Client;

use crate::error::Result;

pub use model::{ContributionCalendar, ContributionDay};

/// Default `User-Agent` sent to GitHub.
pub const DEFAULT_USER_AGENT: &str = concat!("snk-rs/", env!("CARGO_PKG_VERSION"));

/// Client that knows how to talk to the GitHub contribution calendar.
#[derive(Debug, Clone)]
pub struct GitHubClient {
    http: Client,
    user: String,
    token: Option<String>,
    from: Option<NaiveDate>,
    to: Option<NaiveDate>,
    user_agent: String,
}

impl GitHubClient {
    /// Creates a client for the given user.
    ///
    /// When a token is supplied the GraphQL API is used, which gives exact
    /// contribution counts. Without a token the public contribution page is
    /// scraped instead, which only exposes the contribution *levels*.
    pub fn new(user: impl Into<String>, token: Option<String>) -> Self {
        let http = Client::builder()
            .user_agent(DEFAULT_USER_AGENT)
            .build()
            .unwrap_or_default();
        Self {
            http,
            user: user.into(),
            token,
            from: None,
            to: None,
            user_agent: DEFAULT_USER_AGENT.to_string(),
        }
    }

    /// Restricts the calendar to a date range.
    pub fn with_range(mut self, from: Option<NaiveDate>, to: Option<NaiveDate>) -> Self {
        self.from = from;
        self.to = to;
        self
    }

    /// Overrides the `User-Agent` header.
    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = user_agent.into();
        self
    }

    /// Downloads the contribution calendar.
    pub fn fetch_calendar(&self) -> Result<ContributionCalendar> {
        match &self.token {
            Some(token) => graphql::fetch(&self.http, &self.user, token, &self.user_agent),
            None => html::fetch(&self.http, &self.user, self.from, self.to, &self.user_agent),
        }
    }
}
