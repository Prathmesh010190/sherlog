pub mod docker;
pub mod github_actions;
pub mod json_logs;
pub mod kubernetes;
pub mod plain_text;
pub mod syslog;
pub mod terraform;

use serde::{Deserialize, Serialize};

use crate::plugins::{PluginManager, PluginParser};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ParsedLog {
    pub timestamp: Option<String>,
    pub severity: Option<String>,
    pub message: String,
    pub source: Option<String>,
    pub stack_trace: Option<String>,
}

pub trait LogParser: Send + Sync {
    /// Returns the name of this parser.
    fn name(&self) -> &str;

    /// Returns true if this parser can handle the given input.
    fn can_parse(&self, input: &str) -> bool;

    /// Parse the input and return structured log entries.
    fn parse(&self, input: &str) -> Vec<ParsedLog>;
}

/// Detect the best parser for the given input and parse it.
/// Tries WASM plugin parsers first, then falls back to built-in parsers.
pub fn auto_parse(input: &str) -> (String, Vec<ParsedLog>) {
    // Try plugin parsers first
    if let Ok(manager) = PluginManager::new() {
        for mut plugin in manager.load_parser_plugins() {
            if plugin.can_parse(input) {
                let entries = plugin.parse(input);
                if !entries.is_empty() {
                    let name = plugin.parser_name();
                    return (name, entries);
                }
            }
        }
    }

    let parsers: Vec<Box<dyn LogParser>> = vec![
        Box::new(json_logs::JsonLogParser),
        Box::new(kubernetes::KubernetesParser),
        Box::new(docker::DockerParser),
        Box::new(terraform::TerraformParser),
        Box::new(github_actions::GitHubActionsParser),
        Box::new(syslog::SyslogParser),
        Box::new(plain_text::PlainTextParser),
    ];

    for parser in &parsers {
        if parser.can_parse(input) {
            let entries = parser.parse(input);
            if !entries.is_empty() {
                return (parser.name().to_string(), entries);
            }
        }
    }

    // Fallback: return as a single plain text entry
    (
        "plain_text".to_string(),
        vec![ParsedLog {
            message: input.to_string(),
            ..Default::default()
        }],
    )
}
