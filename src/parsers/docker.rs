use super::{LogParser, ParsedLog};
use regex::Regex;

pub struct DockerParser;

impl LogParser for DockerParser {
    fn name(&self) -> &str {
        "docker"
    }

    fn can_parse(&self, input: &str) -> bool {
        let patterns = [
            r"(?i)\b(docker|container|dockerfile|image|layer|COPY|RUN|FROM)\b",
            r"Step \d+/\d+ :",
            r"^#\d+\s+",
            r"--->\s+[0-9a-f]{12}",
        ];

        let hits = input.lines().take(30).filter(|line| {
            patterns
                .iter()
                .any(|p| Regex::new(p).is_ok_and(|re| re.is_match(line)))
        });

        hits.count() >= 2
    }

    fn parse(&self, input: &str) -> Vec<ParsedLog> {
        let mut entries = Vec::new();
        let ts_re = Regex::new(r"^(\d{4}-\d{2}-\d{2}T[\d:.Z]+)\s+(.*)").unwrap();
        let error_re = Regex::new(
            r"(?i)(error|failed|cannot|exit code \d+|no such|permission denied|not found)",
        )
        .unwrap();

        for line in input.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            let (timestamp, message) = if let Some(caps) = ts_re.captures(trimmed) {
                (
                    caps.get(1).map(|m| m.as_str().to_string()),
                    caps.get(2).map_or("", |m| m.as_str()).to_string(),
                )
            } else {
                (None, trimmed.to_string())
            };

            if error_re.is_match(&message) {
                entries.push(ParsedLog {
                    timestamp,
                    severity: Some("error".to_string()),
                    message,
                    source: Some("docker".to_string()),
                    stack_trace: None,
                });
            }
        }

        // If no error lines found, include all lines for context
        if entries.is_empty() {
            for line in input.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    entries.push(ParsedLog {
                        message: trimmed.to_string(),
                        source: Some("docker".to_string()),
                        ..Default::default()
                    });
                }
            }
        }

        entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_docker_build_error() {
        let input = r#"Step 1/5 : FROM node:18-alpine
 ---> abc123def456
Step 2/5 : COPY package.json .
 ---> Using cache
Step 3/5 : RUN npm install
 ---> Running in xyz789
npm ERR! Failed to install dependencies
error: failed to solve: process "npm install" did not complete successfully: exit code: 1"#;

        let parser = DockerParser;
        assert!(parser.can_parse(input));

        let entries = parser.parse(input);
        assert!(!entries.is_empty());
        assert!(entries
            .iter()
            .any(|e| e.severity.as_deref() == Some("error")));
    }

    #[test]
    fn test_not_docker() {
        let parser = DockerParser;
        assert!(!parser.can_parse("just some regular text\nwith multiple lines"));
    }
}
