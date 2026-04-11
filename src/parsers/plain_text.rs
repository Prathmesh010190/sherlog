use super::{LogParser, ParsedLog};
use regex::Regex;

pub struct PlainTextParser;

impl LogParser for PlainTextParser {
    fn name(&self) -> &str {
        "plain_text"
    }

    fn can_parse(&self, _input: &str) -> bool {
        // Plain text is the fallback parser — always returns true
        true
    }

    fn parse(&self, input: &str) -> Vec<ParsedLog> {
        let mut entries = Vec::new();
        let lines: Vec<&str> = input.lines().collect();

        if lines.is_empty() {
            return entries;
        }

        // Try to detect stack traces and group them
        let stack_trace_patterns = [
            // Python
            r"^\s*Traceback \(most recent call last\):",
            r#"^\s*File ".*", line \d+"#,
            // Node.js / JavaScript
            r"^\s+at\s+",
            // Rust
            r"^\s+\d+:\s+0x[0-9a-f]+\s+-\s+",
            r"^thread '.*' panicked at",
            // Java
            r"^\s+at\s+[\w.$]+\([\w.]+:\d+\)",
            r"^([\w.$]+Exception|[\w.$]+Error):",
            // Go
            r"^goroutine\s+\d+\s+\[",
            r"^\s+/.+\.go:\d+",
        ];

        let compiled: Vec<Regex> = stack_trace_patterns
            .iter()
            .filter_map(|p| Regex::new(p).ok())
            .collect();

        let mut current_trace = Vec::new();
        let mut in_trace = false;

        for line in &lines {
            let is_trace_line = compiled.iter().any(|re| re.is_match(line));

            if is_trace_line {
                if !in_trace {
                    in_trace = true;
                }
                current_trace.push(*line);
            } else if in_trace {
                // End of trace
                if !line.trim().is_empty() {
                    current_trace.push(*line);
                }
                let severity = detect_severity(&current_trace.join("\n"));
                entries.push(ParsedLog {
                    message: current_trace.first().unwrap_or(&"").to_string(),
                    stack_trace: Some(current_trace.join("\n")),
                    severity: Some(severity),
                    ..Default::default()
                });
                current_trace.clear();
                in_trace = false;
            }
        }

        // Flush remaining trace
        if !current_trace.is_empty() {
            let severity = detect_severity(&current_trace.join("\n"));
            entries.push(ParsedLog {
                message: current_trace.first().unwrap_or(&"").to_string(),
                stack_trace: Some(current_trace.join("\n")),
                severity: Some(severity),
                ..Default::default()
            });
        }

        // If no stack traces found, return errors/warnings as individual entries
        if entries.is_empty() {
            let error_re =
                Regex::new(r"(?i)\b(error|fatal|panic|exception|fail(ed|ure)?)\b").unwrap();
            let warn_re = Regex::new(r"(?i)\b(warn(ing)?)\b").unwrap();

            for line in &lines {
                if error_re.is_match(line) {
                    entries.push(ParsedLog {
                        message: line.to_string(),
                        severity: Some("error".to_string()),
                        ..Default::default()
                    });
                } else if warn_re.is_match(line) {
                    entries.push(ParsedLog {
                        message: line.to_string(),
                        severity: Some("warning".to_string()),
                        ..Default::default()
                    });
                }
            }
        }

        // Last resort: return the whole input as one entry
        if entries.is_empty() {
            entries.push(ParsedLog {
                message: input.to_string(),
                ..Default::default()
            });
        }

        entries
    }
}

fn detect_severity(text: &str) -> String {
    let lower = text.to_lowercase();
    if lower.contains("fatal") || lower.contains("panic") {
        "fatal".to_string()
    } else if lower.contains("error") || lower.contains("exception") {
        "error".to_string()
    } else if lower.contains("warn") {
        "warning".to_string()
    } else {
        "error".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_python_traceback() {
        let input = r#"Traceback (most recent call last):
  File "app.py", line 10, in <module>
    result = 1 / 0
ZeroDivisionError: division by zero"#;

        let parser = PlainTextParser;
        assert!(parser.can_parse(input));
        let entries = parser.parse(input);
        assert!(!entries.is_empty());
        assert!(entries[0].stack_trace.is_some());
    }

    #[test]
    fn test_nodejs_error() {
        let input = r#"TypeError: Cannot read properties of undefined (reading 'map')
    at Object.<anonymous> (/app/index.js:15:20)
    at Module._compile (node:internal/modules/cjs/loader:1198:14)"#;

        let parser = PlainTextParser;
        let entries = parser.parse(input);
        assert!(!entries.is_empty());
    }

    #[test]
    fn test_rust_panic() {
        let input = r#"thread 'main' panicked at 'index out of bounds: the len is 3 but the index is 5', src/main.rs:10:5
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace"#;

        let parser = PlainTextParser;
        let entries = parser.parse(input);
        assert!(!entries.is_empty());
    }

    #[test]
    fn test_plain_error_line() {
        let input = "error[E0382]: borrow of moved value: `x`";
        let parser = PlainTextParser;
        let entries = parser.parse(input);
        assert!(!entries.is_empty());
        assert_eq!(entries[0].severity.as_deref(), Some("error"));
    }

    #[test]
    fn test_java_exception() {
        let input = r#"java.lang.NullPointerException: Cannot invoke method on null
    at com.example.App.main(App.java:15)
    at sun.reflect.NativeMethodAccessorImpl.invoke0(Native Method)"#;

        let parser = PlainTextParser;
        let entries = parser.parse(input);
        assert!(!entries.is_empty());
    }

    #[test]
    fn test_go_goroutine() {
        let input = r#"goroutine 1 [running]:
main.main()
	/home/user/app/main.go:10 +0x68"#;

        let parser = PlainTextParser;
        let entries = parser.parse(input);
        assert!(!entries.is_empty());
    }
}
