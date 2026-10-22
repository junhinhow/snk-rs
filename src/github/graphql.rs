//! Reads the contribution calendar through the GitHub GraphQL API.
//!
//! This path needs a token. It returns the exact contribution counts and,
//! when the token can read private repositories, private contributions too.

use reqwest::blocking::Client;
use serde::Deserialize;

use crate::error::{Error, Result};
use crate::github::model::{ContributionCalendar, ContributionDay};

const ENDPOINT: &str = "https://api.github.com/graphql";

const QUERY: &str = r#"
query($login: String!) {
  user(login: $login) {
    contributionsCollection {
      contributionCalendar {
        totalContributions
        weeks {
          contributionDays {
            date
            contributionCount
            contributionLevel
          }
        }
      }
    }
  }
}
"#;

/// Fetches the calendar of a user through the GraphQL API.
pub fn fetch(
    http: &Client,
    user: &str,
    token: &str,
    user_agent: &str,
) -> Result<ContributionCalendar> {
    let payload = serde_json::json!({
        "query": QUERY,
        "variables": { "login": user },
    });

    let response = http
        .post(ENDPOINT)
        .header(reqwest::header::USER_AGENT, user_agent)
        .header(reqwest::header::ACCEPT, "application/json")
        .header(reqwest::header::AUTHORIZATION, format!("Bearer {token}"))
        .json(&payload)
        .send()?;

    let status = response.status();
    if status == reqwest::StatusCode::UNAUTHORIZED {
        return Err(Error::NotFound(
            "the GitHub token was rejected (401)".to_string(),
        ));
    }
    if !status.is_success() {
        return Err(Error::InvalidResponse(format!(
            "the GitHub GraphQL API answered with {status}"
        )));
    }

    let body = response.text()?;
    let root: Root = serde_json::from_str(&body)
        .map_err(|error| Error::Parse(format!("invalid GraphQL payload: {error}")))?;

    if let Some(errors) = root.errors {
        let message = errors
            .iter()
            .map(|error| error.message.clone())
            .collect::<Vec<_>>()
            .join("; ");
        return Err(Error::InvalidResponse(format!(
            "the GitHub GraphQL API reported: {message}"
        )));
    }

    let calendar = root
        .data
        .and_then(|data| data.user)
        .ok_or_else(|| Error::NotFound(format!("the user `{user}` was not found")))?
        .contributions_collection
        .contribution_calendar;

    let mut days = Vec::new();
    for week in calendar.weeks {
        for day in week.contribution_days {
            let Ok(date) = chrono::NaiveDate::parse_from_str(&day.date, "%Y-%m-%d") else {
                continue;
            };
            days.push(ContributionDay {
                date,
                count: day.contribution_count,
                level: level_from_name(&day.contribution_level),
            });
        }
    }

    if days.is_empty() {
        return Err(Error::InvalidResponse(
            "the GraphQL API returned an empty calendar".to_string(),
        ));
    }

    let mut result = ContributionCalendar {
        days,
        total: calendar.total_contributions,
    };
    result.normalize();
    Ok(result)
}

/// Translates the GraphQL contribution level into a number from 0 to 4.
fn level_from_name(name: &str) -> u8 {
    match name {
        "FIRST_QUARTILE" => 1,
        "SECOND_QUARTILE" => 2,
        "THIRD_QUARTILE" => 3,
        "FOURTH_QUARTILE" => 4,
        _ => 0,
    }
}

#[derive(Debug, Deserialize)]
struct Root {
    #[serde(default)]
    data: Option<Data>,
    #[serde(default)]
    errors: Option<Vec<GraphQlError>>,
}

#[derive(Debug, Deserialize)]
struct GraphQlError {
    message: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Data {
    #[serde(default)]
    user: Option<User>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct User {
    contributions_collection: ContributionsCollection,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ContributionsCollection {
    contribution_calendar: Calendar,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Calendar {
    total_contributions: i64,
    weeks: Vec<Week>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Week {
    contribution_days: Vec<Day>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Day {
    date: String,
    contribution_count: i64,
    contribution_level: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_levels() {
        assert_eq!(level_from_name("NONE"), 0);
        assert_eq!(level_from_name("FIRST_QUARTILE"), 1);
        assert_eq!(level_from_name("FOURTH_QUARTILE"), 4);
        assert_eq!(level_from_name("unknown"), 0);
    }

    #[test]
    fn parses_graphql_payload() {
        let payload = r#"{
            "data": {
                "user": {
                    "contributionsCollection": {
                        "contributionCalendar": {
                            "totalContributions": 42,
                            "weeks": [
                                {
                                    "contributionDays": [
                                        {"date": "2024-01-01", "contributionCount": 7, "contributionLevel": "THIRD_QUARTILE"}
                                    ]
                                }
                            ]
                        }
                    }
                }
            }
        }"#;
        let root: Root = serde_json::from_str(payload).unwrap();
        let calendar = root.data.unwrap().user.unwrap();
        assert_eq!(
            calendar
                .contributions_collection
                .contribution_calendar
                .total_contributions,
            42
        );
    }
}
