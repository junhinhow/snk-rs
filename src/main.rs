//! Command line entry point.

use std::io::Write;
use std::process::ExitCode;

use clap::Parser;

use snk_rs::cli::Cli;
use snk_rs::error::Result;
use snk_rs::github::stats::ProfileStats;
use snk_rs::github::GitHubClient;
use snk_rs::{render, Grid, RenderOptions};

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(&cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

/// Downloads the calendar, plans the snake and writes the SVG.
fn run(cli: &Cli) -> Result<()> {
    // The calendar is public by default: a profile README shows the public
    // activity. The token is only used for the extra statistics.
    let calendar_token = if cli.include_private {
        cli.token.clone()
    } else {
        None
    };
    let mut client = GitHubClient::new(cli.user.clone(), calendar_token)
        .with_range(cli.from_date()?, cli.to_date()?);
    if let Some(user_agent) = &cli.user_agent {
        client = client.with_user_agent(user_agent.clone());
    }

    let calendar = client.fetch_calendar()?;
    let grid = Grid::from_calendar(&calendar)?;

    let svg = if cli.is_three_dimensional() {
        let mut stats = match &cli.token {
            Some(token) => {
                let http = reqwest::blocking::Client::builder()
                    .user_agent(snk_rs::github::DEFAULT_USER_AGENT)
                    .build()
                    .map_err(snk_rs::Error::Http)?;
                match ProfileStats::fetch(
                    &http,
                    &cli.user,
                    token,
                    snk_rs::github::DEFAULT_USER_AGENT,
                ) {
                    Ok(stats) => stats,
                    Err(error) => {
                        eprintln!("warning: the extra statistics are unavailable ({error})");
                        ProfileStats::from_calendar(&calendar)
                    }
                }
            }
            None => ProfileStats::from_calendar(&calendar),
        };
        stats.period = period_of(&calendar);
        if !cli.include_private {
            // The counter must match the graph, and the graph is public.
            stats.total_contributions = calendar.total;
        }
        snk_rs::render3d::render3d(&grid, &stats, &cli.render3d_options()?)
    } else {
        let plan = snk_rs::snake::plan(&grid);
        let options: RenderOptions = cli.render_options()?;
        render(&grid, &plan, &options)
    };

    match &cli.output {
        Some(path) => {
            if let Some(parent) = path.parent() {
                if !parent.as_os_str().is_empty() {
                    std::fs::create_dir_all(parent)?;
                }
            }
            std::fs::write(path, &svg)?;
            eprintln!(
                "wrote {} ({} bytes, {} contributions, {} active days)",
                path.display(),
                svg.len(),
                grid.total(),
                calendar.active_days()
            );
        }
        None => {
            let stdout = std::io::stdout();
            let mut handle = stdout.lock();
            handle.write_all(svg.as_bytes())?;
            handle.flush()?;
        }
    }

    Ok(())
}

/// The period covered by the calendar, as `start / end`.
fn period_of(calendar: &snk_rs::github::ContributionCalendar) -> String {
    match (
        calendar.days.first().map(|day| day.date),
        calendar.days.last().map(|day| day.date),
    ) {
        (Some(start), Some(end)) => format!("{start} / {end}"),
        _ => String::new(),
    }
}
