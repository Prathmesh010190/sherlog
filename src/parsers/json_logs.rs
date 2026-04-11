use super::{LogParser, ParsedLog};

pub struct JsonLogParser;

impl LogParser for JsonLogParser {
    fn name(&self) -> &str {
        "json"
    }

    fn can_parse(&self, input: &str) -> bool {
        let first_line = input.lines().next().unwrap_or("").trim();
        first_line.starts_with('{') && serde_json::from_str::<serde_json::Value>(first_line).is_ok()
    }

    fn parse(&self, input: &str) -> Vec<ParsedLog> {
        let mut entries = Vec::new();

        for line in input.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            if let Ok(val) = serde_json::from_str::<serde_json::Value>(trimmed) {
                let obj = match val.as_object() {
                    Some(o) => o,
                    None => continue,
                };

                let timestamp =
                    extract_string_field(obj, &["timestamp", "ts", "time", "@timestamp", "t"]);
                let severity =
                    extract_string_field(obj, &["level", "severity", "log_level", "lvl"]);
                let message =
                    extract_string_field(obj, &["message", "msg", "error", "text", "log"]);
                let source = extract_string_field(
                    obj,
                    &["source", "logger", "module", "component", "caller"],
                );
                let stack_trace = extract_string_field(
                    obj,
                    &[
                        "stack_trace",
                        "stacktrace",
                        "stack",
                        "backtrace",
                        "exception",
                    ],
                );

                if let Some(msg) = message {
                    entries.push(ParsedLog {
                        timestamp,
                        severity,
                        message: msg,
                        source,
                        stack_trace,
                    });
                }
            }
        }

        entries
    }
}

fn extract_string_field(
    obj: &serde_json::Map<String, serde_json::Value>,
    keys: &[&str],
) -> Option<String> {
    for key in keys {
        if let Some(val) = obj.get(*key) {
            return match val {
                serde_json::Value::String(s) => Some(s.clone()),
                other => Some(other.to_string()),
            };
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_log_parsing() {
        let input = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"error","message":"Connection refused","source":"db_client"}
{"timestamp":"2024-01-15T10:30:01Z","level":"info","message":"Retrying connection","source":"db_client"}"#;

        let parser = JsonLogParser;
        assert!(parser.can_parse(input));

        let entries = parser.parse(input);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].severity.as_deref(), Some("error"));
        assert_eq!(entries[0].message, "Connection refused");
        assert_eq!(entries[0].source.as_deref(), Some("db_client"));
    }

    #[test]
    fn test_not_json() {
        let parser = JsonLogParser;
        assert!(!parser.can_parse("this is plain text"));
        assert!(!parser.can_parse("ERROR: something failed"));
    }

    #[test]
    fn test_various_field_names() {
        let input =
            r#"{"ts":"2024-01-15","lvl":"warn","msg":"disk space low","caller":"monitor.go:42"}"#;

        let parser = JsonLogParser;
        assert!(parser.can_parse(input));

        let entries = parser.parse(input);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].severity.as_deref(), Some("warn"));
        assert_eq!(entries[0].message, "disk space low");
    }
}
