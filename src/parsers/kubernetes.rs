use super::{LogParser, ParsedLog};
use regex::Regex;

pub struct KubernetesParser;

impl LogParser for KubernetesParser {
    fn name(&self) -> &str {
        "kubernetes"
    }

    fn can_parse(&self, input: &str) -> bool {
        // Kubernetes event log format or kubectl output
        let patterns = [
            r"(?i)\b(pod|deployment|service|namespace|kubectl|kube-system|container)\b",
            r"apiVersion:",
            r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}",
        ];

        let k8s_keywords = input.lines().take(20).filter(|line| {
            patterns
                .iter()
                .any(|p| Regex::new(p).is_ok_and(|re| re.is_match(line)))
        });

        k8s_keywords.count() >= 2
    }

    fn parse(&self, input: &str) -> Vec<ParsedLog> {
        let mut entries = Vec::new();

        // Try parsing kubectl event-style output
        let event_re =
            Regex::new(r"^(\d{4}-\d{2}-\d{2}T[\d:.Z]+)?\s*(\S+)\s+(\S+)\s+(\S+)\s+(.*)").unwrap();

        // Also handle standard k8s log timestamps
        let ts_re = Regex::new(r"^(\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d+Z?)\s+(.*)").unwrap();

        for line in input.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            if let Some(caps) = ts_re.captures(trimmed) {
                let timestamp = caps.get(1).map(|m| m.as_str().to_string());
                let message = caps.get(2).map_or("", |m| m.as_str()).to_string();
                let severity = detect_k8s_severity(&message);

                entries.push(ParsedLog {
                    timestamp,
                    severity: Some(severity),
                    message,
                    source: Some("kubernetes".to_string()),
                    stack_trace: None,
                });
            } else if event_re.is_match(trimmed) {
                let severity = detect_k8s_severity(trimmed);
                entries.push(ParsedLog {
                    timestamp: None,
                    severity: Some(severity),
                    message: trimmed.to_string(),
                    source: Some("kubernetes".to_string()),
                    stack_trace: None,
                });
            }
        }

        // If structured parsing didn't work, just grab error lines
        if entries.is_empty() {
            let error_re = Regex::new(
                r"(?i)(error|warning|failed|crashloopbackoff|oomkilled|evicted|pending)",
            )
            .unwrap();

            for line in input.lines() {
                if error_re.is_match(line) {
                    entries.push(ParsedLog {
                        message: line.trim().to_string(),
                        severity: Some(detect_k8s_severity(line)),
                        source: Some("kubernetes".to_string()),
                        ..Default::default()
                    });
                }
            }
        }

        entries
    }
}

fn detect_k8s_severity(text: &str) -> String {
    let lower = text.to_lowercase();
    if lower.contains("error")
        || lower.contains("fatal")
        || lower.contains("crashloopbackoff")
        || lower.contains("oomkilled")
    {
        "error".to_string()
    } else if lower.contains("warning") || lower.contains("pending") || lower.contains("evicted") {
        "warning".to_string()
    } else {
        "info".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_k8s_log_detection() {
        let input = r#"2024-01-15T10:30:00.123Z Error from pod my-app-xyz: CrashLoopBackOff
2024-01-15T10:30:01.456Z Warning: container failed to start in namespace default"#;

        let parser = KubernetesParser;
        assert!(parser.can_parse(input));

        let entries = parser.parse(input);
        assert!(!entries.is_empty());
    }

    #[test]
    fn test_kubectl_events() {
        let input = r#"LAST SEEN   TYPE      REASON    OBJECT                MESSAGE
5m          Warning   Failed    pod/my-app-abc123     Error: ImagePullBackOff
3m          Normal    Pulling   pod/my-app-abc123     Pulling image "myapp:latest"
2m          Warning   Failed    pod/my-app-abc123     Failed to pull image: container not found"#;

        let parser = KubernetesParser;
        // This has k8s keywords
        let entries = parser.parse(input);
        assert!(!entries.is_empty());
    }
}
