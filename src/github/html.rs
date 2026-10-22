//! Reads the public contribution page of a GitHub user.
//!
//! The page is rendered server side, so the calendar can be read straight from
//! the HTML. This path needs no authentication and therefore only exposes
//! *public* contributions - which is exactly what a profile README should show.

use chrono::NaiveDate;
use regex::Regex;
use reqwest::blocking::Client;

use crate::error::{Error, Result};
use crate::github::model::{estimate_count, ContributionCalendar, ContributionDay};

/// Fetches and parses the public contribution page.
pub fn fetch(
    http: &Client,
    user: &str,
    from: Option<NaiveDate>,
    to: Option<NaiveDate>,
    user_agent: &str,
) -> Result<ContributionCalendar> {
    let mut url = format!("https://github.com/users/{user}/contributions");
    let mut query = Vec::new();
    if let Some(from) = from {
        query.push(format!("from={}", from.format("%Y-%m-%d")));
    }
    if let Some(to) = to {
        query.push(format!("to={}", to.format("%Y-%m-%d")));
    }
    if !query.is_empty() {
        url.push('?');
        url.push_str(&query.join("&"));
    }

    let response = http
        .get(&url)
        .header(reqwest::header::USER_AGENT, user_agent)
        .send()?;

    let status = response.status();
    if status == reqwest::StatusCode::NOT_FOUND {
        return Err(Error::NotFound(format!(
            "the user `{user}` does not exist or has no public contribution page"
        )));
    }
    if !status.is_success() {
        return Err(Error::InvalidResponse(format!(
            "GitHub answered the contribution page with {status}"
        )));
    }

    let body = response.text()?;
    parse(&body)
}

/// Extracts the contribution calendar from the HTML of the contribution page.
pub fn parse(html: &str) -> Result<ContributionCalendar> {
    let cell_pattern = Regex::new(r#"<td[^>]*class="[^"]*ContributionCalendar-day[^"]*"[^>]*>"#)
        .map_err(|error| Error::Parse(error.to_string()))?;
    let date_pattern = Regex::new(r#"data-date="(\d{4}-\d{2}-\d{2})""#)
        .map_err(|error| Error::Parse(error.to_string()))?;
    let level_pattern =
        Regex::new(r#"data-level="(\d)""#).map_err(|error| Error::Parse(error.to_string()))?;

    let mut days = Vec::new();
    for cell in cell_pattern.find_iter(html) {
        let tag = cell.as_str();
        let Some(date_match) = date_pattern.captures(tag) else {
            continue;
        };
        let Some(level_match) = level_pattern.captures(tag) else {
            continue;
        };
        let Ok(date) = NaiveDate::parse_from_str(&date_match[1], "%Y-%m-%d") else {
            continue;
        };
        let level: u8 = level_match[1].parse().unwrap_or(0);
        days.push(ContributionDay {
            date,
            count: estimate_count(level),
            level,
        });
    }

    if days.is_empty() {
        return Err(Error::Parse(
            "no contribution cells found in the response".to_string(),
        ));
    }

    let mut calendar = ContributionCalendar { days, total: 0 };
    calendar.normalize();
    Ok(calendar)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
        <table>
          <td tabindex="-1" data-ix="0.0" class="ContributionCalendar-day" data-date="2024-01-01" data-level="0"></td>
          <td tabindex="-1" data-ix="1.0" class="ContributionCalendar-day" data-date="2024-01-02" data-level="2"></td>
          <td tabindex="-1" data-ix="2.0" class="ContributionCalendar-day" data-date="2024-01-03" data-level="4"></td>
        </table>
    "#;

    #[test]
    fn parses_days_and_levels() {
        let calendar = parse(SAMPLE).unwrap();
        assert_eq!(calendar.days.len(), 3);
        assert_eq!(calendar.days[0].level, 0);
        assert_eq!(calendar.days[1].level, 2);
        assert_eq!(calendar.days[2].level, 4);
        assert_eq!(calendar.days[2].count, 10);
        assert_eq!(calendar.active_days(), 2);
    }

    #[test]
    fn rejects_html_without_cells() {
        assert!(parse("<html><body>nothing here</body></html>").is_err());
    }
}
