use super::{LogParser, ParsedLog};
use regex::Regex;

pub struct SyslogParser;

impl LogParser for SyslogParser {
    fn name(&self) -> &str {
        "syslog"
    }

    fn can_parse(&self, input: &str) -> bool {
        let re = Regex::new(r"^[A-Z][a-z]{2}\s+\d{1,2}\s+\d{2}:\d{2}:\d{2}\s+\S+\s+\S+").unwrap();
        input.lines().take(5).any(|line| re.is_match(line.trim()))
    }

    fn parse(&self, input: &str) -> Vec<ParsedLog> {
        let re = Regex::new(
            r"^([A-Z][a-z]{2}\s+\d{1,2}\s+\d{2}:\d{2}:\d{2})\s+(\S+)\s+(\S+?)(?:\[\d+\])?:\s*(.*)",
        )
        .unwrap();

        let mut entries = Vec::new();

        for line in input.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            if let Some(caps) = re.captures(trimmed) {
                let timestamp = caps.get(1).map(|m| m.as_str().to_string());
                let source = Some(caps.get(3).map_or("", |m| m.as_str()).to_string());
                let message = caps.get(4).map_or("", |m| m.as_str()).to_string();
                let severity = detect_syslog_severity(&message);

                entries.push(ParsedLog {
                    timestamp,
                    severity: Some(severity),
                    message,
                    source,
                    stack_trace: None,
                });
            }
        }

        entries
    }
}

fn detect_syslog_severity(msg: &str) -> String {
    let lower = msg.to_lowercase();
    if lower.contains("error") || lower.contains("fatal") || lower.contains("crit") {
        "error".to_string()
    } else if lower.contains("warn") {
        "warning".to_string()
    } else if lower.contains("debug") {
        "debug".to_string()
    } else {
        "info".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syslog_parsing() {
        let input = r#"Jan 15 10:30:00 myhost sshd[1234]: Failed password for root from 192.168.1.1 port 22 ssh2
Jan 15 10:30:01 myhost kernel: Out of memory: Kill process 5678"#;

        let parser = SyslogParser;
        assert!(parser.can_parse(input));

        let entries = parser.parse(input);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].source.as_deref(), Some("sshd"));
        assert!(entries[0].message.contains("Failed password"));
    }

    #[test]
    fn test_not_syslog() {
        let parser = SyslogParser;
        assert!(!parser.can_parse("just regular text"));
        assert!(!parser.can_parse(r#"{"level": "error"}"#));
    }
}
