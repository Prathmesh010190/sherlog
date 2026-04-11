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

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        output::print_error(&format!("{e:#}"));
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let cli = Cli::parse();

    if cli.no_color {
        colored::control::set_override(false);
    }

    match &cli.command {
        None => commands::default::run(cli.json).await,
        Some(Commands::Init { shell }) => commands::init::run(shell.clone()),
        Some(Commands::Analyze { file, max_lines }) => {
            commands::analyze::run(file, *max_lines, cli.json).await
        }
        Some(Commands::Explain { error_message }) => {
            commands::explain::run(error_message, cli.json).await
        }
        Some(Commands::History { last }) => commands::history::run(*last, cli.json),
        Some(Commands::Config { action }) => commands::config::run(action, cli.json),
        Some(Commands::Pipe) => commands::pipe::run(cli.json).await,
    }
}
