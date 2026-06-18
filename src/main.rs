mod ai;
mod cli;
mod commands;
mod config;
mod context;
mod output;
mod parsers;
mod plugins;
mod storage;
mod telemetry;

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

    // Record anonymous telemetry if opted in
    if config::Config::load().map(|c| c.telemetry).unwrap_or(false) {
        let cmd = match &cli.command {
            None => "default",
            Some(Commands::Init { .. }) => "init",
            Some(Commands::Analyze { .. }) => "analyze",
            Some(Commands::Explain { .. }) => "explain",
            Some(Commands::History { .. }) => "history",
            Some(Commands::Config { .. }) => "config",
            Some(Commands::Pipe) => "pipe",
            Some(Commands::Postmortem { .. }) => "postmortem",
            Some(Commands::Patterns { .. }) => "patterns",
            Some(Commands::Watch { .. }) => "watch",
            Some(Commands::Plugin { .. }) => "plugin",
            Some(Commands::Trace { .. }) => "trace",
        };
        telemetry::Telemetry::new().record_command(cmd);
    }

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
        Some(Commands::Plugin { action }) => commands::plugin::run(action, mode).await,
        Some(Commands::Trace {
            services,
            since,
            files,
        }) => commands::trace::run(services, since, files, mode).await,
    }
}
