use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "sherlog",
    about = "A developer-focused error debugging assistant for the terminal",
    version,
    long_about = "Sherlog captures failed commands, analyzes log files, and explains errors using AI — right from your command line."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Output results as JSON
    #[arg(long, global = true)]
    pub json: bool,

    /// Disable colored output
    #[arg(long, global = true)]
    pub no_color: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Install shell hook for automatic error capture
    Init {
        /// Shell type to generate hook for (bash, zsh, fish). Auto-detected if not specified.
        #[arg(long)]
        shell: Option<String>,
    },

    /// Analyze a log file
    Analyze {
        /// Path to the log file
        file: String,

        /// Maximum number of lines to process
        #[arg(long)]
        max_lines: Option<usize>,
    },

    /// Explain an error message directly
    Explain {
        /// The error message to explain
        error_message: Vec<String>,
    },

    /// Show past error analyses from local history
    History {
        /// Number of recent entries to show
        #[arg(long, default_value = "10")]
        last: usize,
    },

    /// Manage configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// Read from stdin for piped input
    Pipe,
}

#[derive(Subcommand)]
pub enum ConfigAction {
    /// Set a configuration value
    Set {
        /// Configuration key (api_key, model, max_log_lines, theme, api_base_url)
        key: String,
        /// Value to set
        value: String,
    },

    /// Show current configuration
    Show,
}
