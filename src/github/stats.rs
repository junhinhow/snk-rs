//! Extra statistics about a user: the numbers behind the radar chart, the
//! language breakdown and the counters printed under the 3D graph.

use reqwest::blocking::Client;
use serde::Deserialize;

use crate::error::{Error, Result};
use crate::github::model::ContributionCalendar;

/// The five axes of the radar chart.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    /// Commits.
    Commit,
    /// Issues.
    Issue,
    /// Pull requests.
    PullRequest,
    /// Pull request reviews.
    Review,
    /// Repositories contributed to.
    Repository,
}

impl Axis {
    /// All axes, in the order they are drawn.
    pub const ALL: [Axis; 5] = [
        Axis::Commit,
        Axis::Issue,
        Axis::PullRequest,
        Axis::Review,
        Axis::Repository,
    ];

    /// The label drawn next to the axis.
    pub fn label(self) -> &'static str {
        match self {
            Axis::Commit => "Commit",
            Axis::Issue => "Issue",
            Axis::PullRequest => "PullReq",
            Axis::Review => "Review",
            Axis::Repository => "Repo",
        }
    }

    /// The value of this axis.
    pub fn value(self, stats: &ProfileStats) -> u64 {
        match self {
            Axis::Commit => stats.commits,
            Axis::Issue => stats.issues,
            Axis::PullRequest => stats.pull_requests,
            Axis::Review => stats.reviews,
            Axis::Repository => stats.repositories,
        }
    }
}

/// A language and how many bytes of it the user has written.
#[derive(Debug, Clone, PartialEq)]
pub struct Language {
    /// Name of the language.
    pub name: String,
    /// Colour GitHub assigns to the language.
    pub color: String,
    /// Size in bytes.
    pub size: u64,
}

/// Everything the renderer needs besides the contribution calendar.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ProfileStats {
    /// Total contributions in the period.
    pub total_contributions: i64,
    /// Commits.
    pub commits: u64,
    /// Issues.
    pub issues: u64,
    /// Pull requests.
    pub pull_requests: u64,
    /// Pull request reviews.
    pub reviews: u64,
    /// Repositories the user contributed to.
    pub repositories: u64,
    /// Stars received by the user's repositories.
    pub stars: u64,
    /// Number of public repositories.
    pub repository_count: u64,
    /// The period covered by the graph, as `start / end`.
    pub period: String,
    /// The language breakdown, biggest first.
    pub languages: Vec<Language>,
}

impl ProfileStats {
    /// Fetches the statistics from the GraphQL API.
    pub fn fetch(http: &Client, user: &str, token: &str, user_agent: &str) -> Result<Self> {
        let payload = serde_json::json!({
            "query": QUERY,
            "variables": { "login": user },
        });

        let response = http
            .post("https://api.github.com/graphql")
            .header(reqwest::header::USER_AGENT, user_agent)
            .header(reqwest::header::ACCEPT, "application/json")
            .header(reqwest::header::AUTHORIZATION, format!("Bearer {token}"))
            .json(&payload)
            .send()?;

        let status = response.status();
        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(Error::NotFound(
                "the GitHub token was rejected (401)".into(),
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

        let user = root
            .data
            .and_then(|data| data.user)
            .ok_or_else(|| Error::NotFound(format!("the user `{user}` was not found")))?;

        let contributions = user.contributions_collection;
        let mut languages: Vec<Language> = Vec::new();
        let mut stars = 0u64;

        for repository in user.repositories.nodes {
            stars += repository.stargazer_count;
            for edge in repository.languages.edges {
                let Some(color) = edge.node.color.clone() else {
                    continue;
                };
                if let Some(existing) = languages.iter_mut().find(|l| l.name == edge.node.name) {
                    existing.size += edge.size;
                } else {
                    languages.push(Language {
                        name: edge.node.name.clone(),
                        color,
                        size: edge.size,
                    });
                }
            }
        }

        languages.sort_by_key(|language| std::cmp::Reverse(language.size));

        Ok(Self {
            total_contributions: contributions.contribution_calendar.total_contributions,
            commits: contributions.total_commit_contributions,
            issues: contributions.total_issue_contributions,
            pull_requests: contributions.total_pull_request_contributions,
            reviews: contributions.total_pull_request_review_contributions,
            repositories: user.repositories_contributed_to.total_count,
            stars,
            repository_count: user.repositories.total_count,
            period: String::new(),
            languages,
        })
    }

    /// Builds the statistics from a calendar only, used when no token is
    /// available. The radar and the donut stay empty in that case.
    pub fn from_calendar(calendar: &ContributionCalendar) -> Self {
        Self {
            total_contributions: calendar.total,
            ..Self::default()
        }
    }
}

const QUERY: &str = r#"
query($login: String!) {
  user(login: $login) {
    contributionsCollection {
      contributionCalendar {
        totalContributions
      }
      totalCommitContributions
      totalIssueContributions
      totalPullRequestContributions
      totalPullRequestReviewContributions
    }
    repositoriesContributedTo(first: 1) {
      totalCount
    }
    repositories(first: 100, orderBy: {field: STARGAZERS, direction: DESC}, ownerAffiliations: OWNER) {
      totalCount
      nodes {
        stargazerCount
        languages(first: 10, orderBy: {field: SIZE, direction: DESC}) {
          edges {
            size
            node {
              name
              color
            }
          }
        }
      }
    }
  }
}
"#;

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
    contributions_collection: Contributions,
    repositories_contributed_to: Contributed,
    repositories: Repositories,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Contributions {
    contribution_calendar: Calendar,
    total_commit_contributions: u64,
    total_issue_contributions: u64,
    total_pull_request_contributions: u64,
    total_pull_request_review_contributions: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Calendar {
    total_contributions: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Contributed {
    #[serde(default)]
    total_count: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Repositories {
    #[serde(default)]
    total_count: u64,
    #[serde(default)]
    nodes: Vec<Repository>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Repository {
    #[serde(default)]
    stargazer_count: u64,
    #[serde(default)]
    languages: Languages,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Languages {
    #[serde(default)]
    edges: Vec<Edge>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Edge {
    #[serde(default)]
    size: u64,
    node: Node,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Node {
    name: String,
    #[serde(default)]
    color: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn axes_have_labels() {
        assert_eq!(Axis::ALL.len(), 5);
        assert_eq!(Axis::PullRequest.label(), "PullReq");
    }

    #[test]
    fn stats_from_calendar_keep_the_total() {
        let calendar = ContributionCalendar {
            days: Vec::new(),
            total: 42,
        };
        let stats = ProfileStats::from_calendar(&calendar);
        assert_eq!(stats.total_contributions, 42);
        assert_eq!(stats.commits, 0);
    }
}
