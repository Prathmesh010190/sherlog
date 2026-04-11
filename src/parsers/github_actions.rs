use super::{LogParser, ParsedLog};
use regex::Regex;

pub struct GitHubActionsParser;

impl LogParser for GitHubActionsParser {
    fn name(&self) -> &str {
        "github_actions"
    }

    fn can_parse(&self, input: &str) -> bool {
        let patterns = [
            r"^##\[error\]",
            r"^##\[warning\]",
            r"^##\[group\]",
            r"^##\[endgroup\]",
            r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d+Z\s+",
            r"(?i)Run\s+[\w-]+/[\w-]+@",
            r"::error[\s:]",
            r"::warning[\s:]",
        ];

        let hits = input.lines().take(30).filter(|line| {
            patterns
                .iter()
                .any(|p| Regex::new(p).is_ok_and(|re| re.is_match(line.trim())))
        });

        hits.count() >= 2
    }

    fn parse(&self, input: &str) -> Vec<ParsedLog> {
        let mut entries = Vec::new();
        let annotation_re = Regex::new(r"^##\[(error|warning|notice|debug)\](.*)").unwrap();
        let workflow_cmd_re =
            Regex::new(r"::(error|warning|notice|debug)(?:\s+[^:]*)?::(.*)").unwrap();
        let ts_re = Regex::new(r"^(\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d+Z)\s+(.*)").unwrap();

        for line in input.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            if let Some(caps) = annotation_re.captures(trimmed) {
                let severity = caps.get(1).map_or("info", |m| m.as_str()).to_string();
                let message = caps.get(2).map_or("", |m| m.as_str()).trim().to_string();

                entries.push(ParsedLog {
                    severity: Some(severity),
                    message,
                    source: Some("github_actions".to_string()),
                    ..Default::default()
                });
            } else if let Some(caps) = workflow_cmd_re.captures(trimmed) {
                let severity = caps.get(1).map_or("info", |m| m.as_str()).to_string();
                let message = caps.get(2).map_or("", |m| m.as_str()).trim().to_string();

                entries.push(ParsedLog {
                    severity: Some(severity),
                    message,
                    source: Some("github_actions".to_string()),
                    ..Default::default()
                });
            } else if let Some(caps) = ts_re.captures(trimmed) {
                let timestamp = caps.get(1).map(|m| m.as_str().to_string());
                let message = caps.get(2).map_or("", |m| m.as_str()).to_string();
                let severity = if message.to_lowercase().contains("error") {
                    "error"
                } else if message.to_lowercase().contains("warning") {
                    "warning"
                } else {
                    "info"
                };

                entries.push(ParsedLog {
                    timestamp,
                    severity: Some(severity.to_string()),
                    message,
                    source: Some("github_actions".to_string()),
                    stack_trace: None,
                });
            }
        }

        entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_github_actions_annotations() {
        let input = r#"##[group]Run actions/checkout@v4
##[endgroup]
##[error]Process completed with exit code 1.
##[warning]Node.js 16 actions are deprecated."#;

        let parser = GitHubActionsParser;
        assert!(parser.can_parse(input));

        let entries = parser.parse(input);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].severity.as_deref(), Some("error"));
        assert_eq!(entries[1].severity.as_deref(), Some("warning"));
    }

    #[test]
    fn test_workflow_commands() {
        let input = r#"::error file=app.js,line=1::Missing semicolon
::warning file=test.js,line=10::Test is flaky"#;

        let parser = GitHubActionsParser;
        assert!(parser.can_parse(input));

        let entries = parser.parse(input);
        assert_eq!(entries.len(), 2);
    }
}
