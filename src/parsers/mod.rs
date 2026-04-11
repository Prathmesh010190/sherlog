pub mod docker;
pub mod github_actions;
pub mod json_logs;
pub mod kubernetes;
pub mod plain_text;
pub mod syslog;
pub mod terraform;

use serde::{Deserialize, Serialize};

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
pub fn auto_parse(input: &str) -> (String, Vec<ParsedLog>) {
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
