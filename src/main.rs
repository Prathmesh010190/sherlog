mod ai;
mod cli;
mod commands;
mod config;
mod context;
mod output;
mod parsers;
mod storage;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};
use output::OutputMode;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        output::print_error(&format!("{e:#}"));
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let cli = Cli::parse();

    if cli.no_color || cli.ci {
        colored::control::set_override(false);
    }

    let mode = OutputMode::from_flags(cli.json, cli.ci);

    match &cli.command {
        None => commands::default::run(mode).await,
        Some(Commands::Init { shell }) => commands::init::run(shell.clone()),
        Some(Commands::Analyze { file, max_lines }) => {
            commands::analyze::run(file, *max_lines, mode).await
        }
        Some(Commands::Explain { error_message }) => {
            commands::explain::run(error_message, mode).await
        }
        Some(Commands::History { last }) => commands::history::run(*last, mode),
        Some(Commands::Config { action }) => commands::config::run(action, mode),
        Some(Commands::Pipe) => commands::pipe::run(mode).await,
        Some(Commands::Postmortem { since }) => commands::postmortem::run(since, mode).await,
        Some(Commands::Patterns { last }) => commands::patterns::run(last, mode),
        Some(Commands::Watch { path }) => commands::watch::run(path, mode).await,
    }
}
