//! Command line entry point.

use std::io::Write;
use std::process::ExitCode;

use clap::Parser;

use snk_rs::cli::Cli;
use snk_rs::error::Result;
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
    let mut client = GitHubClient::new(cli.user.clone(), cli.token.clone())
        .with_range(cli.from_date()?, cli.to_date()?);
    if let Some(user_agent) = &cli.user_agent {
        client = client.with_user_agent(user_agent.clone());
    }

    let calendar = client.fetch_calendar()?;
    let grid = Grid::from_calendar(&calendar)?;
    let plan = snk_rs::snake::plan(&grid);
    let options: RenderOptions = cli.render_options()?;
    let svg = render(&grid, &plan, &options);

    match &cli.output {
        Some(path) => {
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
