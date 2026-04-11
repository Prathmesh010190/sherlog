use extism_pdk::*;
use serde::Serialize;

#[derive(Serialize)]
struct ParsedLog {
    timestamp: Option<String>,
    severity: Option<String>,
    message: String,
    source: Option<String>,
    stack_trace: Option<String>,
}

/// Return the parser name.
#[plugin_fn]
pub fn name(_input: String) -> FnResult<String> {
    Ok("example-csv-parser".to_string())
}

/// Check if this parser can handle the input.
/// Detects simple CSV log format: timestamp,severity,message
#[plugin_fn]
pub fn can_parse(input: String) -> FnResult<String> {
    let first_line = input.lines().next().unwrap_or("");
    let parts: Vec<&str> = first_line.splitn(3, ',').collect();
    if parts.len() >= 3 {
        let severity = parts[1].trim().to_lowercase();
        if ["error", "warn", "warning", "info", "debug", "fatal"]
            .contains(&severity.as_str())
        {
            return Ok("true".to_string());
        }
    }
    Ok("false".to_string())
}

/// Parse CSV log lines into structured JSON entries.
/// Expected format per line: timestamp,severity,message
#[plugin_fn]
pub fn parse(input: String) -> FnResult<String> {
    let mut entries = Vec::new();
    for line in input.lines() {
        let parts: Vec<&str> = line.splitn(3, ',').collect();
        if parts.len() >= 3 {
            entries.push(ParsedLog {
                timestamp: Some(parts[0].trim().to_string()),
                severity: Some(parts[1].trim().to_lowercase()),
                message: parts[2].trim().to_string(),
                source: None,
                stack_trace: None,
            });
        }
    }
    Ok(serde_json::to_string(&entries).unwrap_or_else(|_| "[]".to_string()))
}
