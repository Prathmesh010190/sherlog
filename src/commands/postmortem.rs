use anyhow::{bail, Result};
use colored::Colorize;

use crate::ai::AiEngine;
use crate::config::Config;
use crate::output::{self, OutputMode};
use crate::storage::Storage;

const POSTMORTEM_SYSTEM_PROMPT: &str = r#"You are a senior SRE writing an incident postmortem report. Given a list of errors, their root causes, and fixes from a developer's recent shell history, generate a clear, professional markdown postmortem report.

The report must include:
1. **Incident Summary** — one-paragraph overview
2. **Timeline** — chronological list of errors with timestamps
3. **Root Causes** — deduplicated list of underlying issues
4. **Impact** — what was affected
5. **Resolution** — what was done to fix each issue
6. **Lessons Learned** — what to improve
7. **Action Items** — concrete follow-up tasks with owners (mark as [ ] TODO)

Keep it concise and actionable. Use markdown formatting."#;

pub async fn run(since: &str, mode: OutputMode) -> Result<()> {
    let since_datetime = parse_since(since)?;

    let storage = Storage::open()?;
    let entries = storage.get_since(&since_datetime)?;

    if entries.is_empty() {
        if mode == OutputMode::Json {
            println!(r#"{{"error": "No errors found in the specified time window"}}"#);
        } else {
            output::print_info(&format!(
                "No errors found since {since_datetime}. Nothing to report."
            ));
        }
        return Ok(());
    }

    if mode == OutputMode::Terminal {
        output::print_info(&format!(
            "Generating postmortem from {} error(s) since {since_datetime}...",
            entries.len()
        ));
        println!();
    }

    // Build a prompt from the error history
    let mut prompt = format!(
        "Generate a postmortem report for the following {} errors:\n\n",
        entries.len()
    );

    for entry in &entries {
        prompt.push_str(&format!("---\nTimestamp: {}\n", entry.timestamp));
        prompt.push_str(&format!("Command: {}\n", entry.command));
        prompt.push_str(&format!("Root Cause: {}\n", entry.root_cause));

        if let Ok(fixes) = serde_json::from_str::<Vec<serde_json::Value>>(&entry.fixes_json) {
            for fix in &fixes {
                if let Some(desc) = fix.get("description").and_then(|d| d.as_str()) {
                    prompt.push_str(&format!("Fix: {}\n", desc));
                }
            }
        }

        // Include a truncated error snippet
        let error_preview = if entry.error_text.len() > 500 {
            format!("{}...", &entry.error_text[..500])
        } else {
            entry.error_text.clone()
        };
        prompt.push_str(&format!("Error: {}\n", error_preview));
    }

    let config = Config::load()?;
    let engine = AiEngine::new(config)?;
    let report = engine
        .generate_text(POSTMORTEM_SYSTEM_PROMPT, &prompt)
        .await?;

    match mode {
        OutputMode::Json => {
            let json = serde_json::json!({
                "since": since_datetime,
                "error_count": entries.len(),
                "report": report,
            });
            println!("{}", serde_json::to_string_pretty(&json)?);
        }
        OutputMode::Ci => {
            println!("::group::Sherlog Postmortem Report");
            println!("{report}");
            println!("::endgroup::");
        }
        OutputMode::Terminal => {
            println!("{}", "Incident Postmortem".cyan().bold().underline());
            println!();
            println!("{report}");
        }
    }

    Ok(())
}

/// Parse a human-friendly "since" string into a SQLite datetime string.
/// Supports: "2h ago", "30m ago", "1d ago", "2h", "30m", "1d"
pub fn parse_since(since: &str) -> Result<String> {
    let s = since.trim().trim_end_matches("ago").trim();

    let re = regex::Regex::new(r"^(\d+)\s*(h|m|d|hr|min|hour|day|hours|minutes|days)s?$")?;
    let caps = re.captures(s).ok_or_else(|| {
        anyhow::anyhow!("Invalid time format: '{since}'. Use e.g. '2h ago', '30m ago', '1d ago'")
    })?;

    let amount: i64 = caps[1].parse()?;
    let unit = &caps[2];

    let seconds = match unit {
        "m" | "min" | "minute" | "minutes" => amount * 60,
        "h" | "hr" | "hour" | "hours" => amount * 3600,
        "d" | "day" | "days" => amount * 86400,
        _ => bail!("Unknown time unit: '{unit}'"),
    };

    let since_time = chrono::Utc::now() - chrono::Duration::seconds(seconds);
    Ok(since_time.format("%Y-%m-%d %H:%M:%S").to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_since_hours() {
        let result = parse_since("2h ago");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_since_minutes() {
        let result = parse_since("30m ago");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_since_days() {
        let result = parse_since("1d ago");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_since_no_ago() {
        let result = parse_since("2h");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_since_invalid() {
        let result = parse_since("potato");
        assert!(result.is_err());
    }
}
