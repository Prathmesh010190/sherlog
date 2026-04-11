use super::{LogParser, ParsedLog};
use regex::Regex;

pub struct TerraformParser;

impl LogParser for TerraformParser {
    fn name(&self) -> &str {
        "terraform"
    }

    fn can_parse(&self, input: &str) -> bool {
        let patterns = [
            r"(?i)\bterraform\b",
            r"(?i)\b(Error|Warning):\s+",
            r"on\s+\S+\.tf\s+line\s+\d+",
            r#"resource\s+""#,
            r#"provider\s+""#,
            r"module\.",
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
        let error_block_re = Regex::new(r"(?i)^\u{2577}?\s*(Error|Warning):\s*(.*)").unwrap();
        let location_re = Regex::new(r"on\s+(\S+\.tf)\s+line\s+(\d+)").unwrap();

        let lines: Vec<&str> = input.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i].trim();

            if let Some(caps) = error_block_re.captures(line) {
                let severity = caps.get(1).map_or("error", |m| m.as_str()).to_lowercase();
                let mut message = caps.get(2).map_or("", |m| m.as_str()).to_string();

                // Collect the rest of the error block
                let mut block_lines = vec![line.to_string()];
                let mut source = None;
                i += 1;

                while i < lines.len() {
                    let next = lines[i].trim();
                    if next.is_empty() || error_block_re.is_match(next) {
                        break;
                    }
                    block_lines.push(next.to_string());

                    if let Some(loc_caps) = location_re.captures(next) {
                        source = Some(format!(
                            "{}:{}",
                            loc_caps.get(1).map_or("", |m| m.as_str()),
                            loc_caps.get(2).map_or("", |m| m.as_str())
                        ));
                    }

                    i += 1;
                }

                if message.is_empty() && block_lines.len() > 1 {
                    message = block_lines[1..].join(" ").trim().to_string();
                }

                entries.push(ParsedLog {
                    timestamp: None,
                    severity: Some(severity),
                    message,
                    source,
                    stack_trace: Some(block_lines.join("\n")),
                });
            } else {
                i += 1;
            }
        }

        entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terraform_error() {
        let input = r#"
Error: Reference to undeclared resource

  on main.tf line 15, in resource "aws_instance" "web":
  15:   ami = aws_ami.ubuntu.id

A managed resource "aws_ami" "ubuntu" has not been declared in the root module.

Warning: Deprecated attribute

  on main.tf line 20, in resource "aws_instance" "web":
  20:   associate_public_ip_address = true

This attribute is deprecated. Use network_interface instead.
"#;

        let parser = TerraformParser;
        assert!(parser.can_parse(input));

        let entries = parser.parse(input);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].severity.as_deref(), Some("error"));
        assert_eq!(entries[1].severity.as_deref(), Some("warning"));
    }
}
