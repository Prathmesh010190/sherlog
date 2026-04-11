use anyhow::{bail, Context, Result};
use std::fs;
use std::path::Path;

use crate::ai::AiEngine;
use crate::config::Config;
use crate::context::ProjectContext;
use crate::output;
use crate::parsers;
use crate::storage::Storage;

const MAX_FILE_SIZE: u64 = 50 * 1024 * 1024; // 50MB
const CHUNK_SIZE: usize = 100_000; // ~100KB per chunk

pub async fn run(file: &str, max_lines: Option<usize>, json_output: bool) -> Result<()> {
    let path = Path::new(file);
    if !path.exists() {
        bail!("File not found: {}", file);
    }

    let metadata = fs::metadata(path).context("Failed to read file metadata")?;
    if metadata.len() > MAX_FILE_SIZE {
        bail!(
            "File too large ({:.1}MB). Maximum supported size is 50MB.",
            metadata.len() as f64 / 1024.0 / 1024.0
        );
    }

    let raw = fs::read(path).context("Failed to read file")?;
    let stripped = strip_ansi_escapes::strip(&raw);
    let content = String::from_utf8_lossy(&stripped).to_string();

    // Apply line limit
    let config = Config::load()?;
    let line_limit = max_lines.unwrap_or(config.max_log_lines);
    let lines: Vec<&str> = content.lines().collect();

    let truncated = if lines.len() > line_limit {
        let selected: Vec<&str> = lines[lines.len() - line_limit..].to_vec();
        if !json_output {
            output::print_info(&format!(
                "File has {} lines. Analyzing the last {} lines.",
                lines.len(),
                line_limit
            ));
        }
        selected.join("\n")
    } else {
        content.clone()
    };

    // Auto-detect format and parse
    let (format_name, parsed) = parsers::auto_parse(&truncated);

    if !json_output {
        output::print_info(&format!("Detected format: {format_name}"));
    }

    let context = ProjectContext::detect();

    if !json_output {
        output::print_context_info(&context.project_types);
    }

    // For large files, chunk and summarize
    let error_text = if truncated.len() > CHUNK_SIZE {
        build_chunked_prompt(&truncated, &format_name, &parsed)
    } else {
        build_prompt(&truncated, &format_name, &parsed)
    };

    let engine = AiEngine::new(config)?;
    let response = engine.analyze(&error_text, Some(&context)).await?;

    // Store in history
    if let Ok(storage) = Storage::open() {
        let fixes_json = serde_json::to_string(&response.fixes).unwrap_or_default();
        let _ = storage.insert(
            &format!("sherlog analyze {file}"),
            &truncated[..truncated.len().min(10000)],
            &response.root_cause,
            &fixes_json,
            &context.summary(),
        );
    }

    output::print_analysis(&response, json_output);
    Ok(())
}

fn build_prompt(content: &str, format_name: &str, parsed: &[parsers::ParsedLog]) -> String {
    let mut prompt = format!("Log file analysis (format: {format_name}):\n\n");

    // Include parsed entries summary
    let error_entries: Vec<_> = parsed
        .iter()
        .filter(|e| {
            e.severity
                .as_deref()
                .is_some_and(|s| s == "error" || s == "fatal")
        })
        .collect();

    if !error_entries.is_empty() {
        prompt.push_str(&format!(
            "Found {} error entries out of {} total.\n\n",
            error_entries.len(),
            parsed.len()
        ));
        prompt.push_str("Error entries:\n");
        for entry in error_entries.iter().take(50) {
            if let Some(ts) = &entry.timestamp {
                prompt.push_str(&format!("[{ts}] "));
            }
            prompt.push_str(&entry.message);
            if let Some(trace) = &entry.stack_trace {
                prompt.push_str(&format!("\n{trace}"));
            }
            prompt.push('\n');
        }
        prompt.push('\n');
    }

    prompt.push_str("Full log content:\n");
    prompt.push_str(content);
    prompt
}

fn build_chunked_prompt(content: &str, format_name: &str, parsed: &[parsers::ParsedLog]) -> String {
    let mut prompt = format!(
        "Large log file analysis (format: {format_name}, {} total entries):\n\n",
        parsed.len()
    );

    // Summarize the parsed entries
    let error_entries: Vec<_> = parsed
        .iter()
        .filter(|e| {
            e.severity
                .as_deref()
                .is_some_and(|s| s == "error" || s == "fatal")
        })
        .collect();

    let warning_entries: Vec<_> = parsed
        .iter()
        .filter(|e| e.severity.as_deref() == Some("warning"))
        .collect();

    prompt.push_str(&format!(
        "Summary: {} errors, {} warnings\n\n",
        error_entries.len(),
        warning_entries.len()
    ));

    // Include all error entries
    if !error_entries.is_empty() {
        prompt.push_str("Error entries:\n");
        for entry in error_entries.iter().take(100) {
            if let Some(ts) = &entry.timestamp {
                prompt.push_str(&format!("[{ts}] "));
            }
            prompt.push_str(&entry.message);
            if let Some(trace) = &entry.stack_trace {
                prompt.push_str(&format!("\n{trace}"));
            }
            prompt.push('\n');
        }
        prompt.push('\n');
    }

    // Include a sample of the raw content (first and last sections)
    let max_raw = CHUNK_SIZE;
    if content.len() > max_raw {
        let half = max_raw / 2;
        prompt.push_str("Log content (truncated, showing start and end):\n\n--- START ---\n");
        prompt.push_str(&content[..half]);
        prompt.push_str("\n\n--- ... ---\n\n");
        prompt.push_str(&content[content.len() - half..]);
        prompt.push_str("\n--- END ---\n");
    } else {
        prompt.push_str("Full log content:\n");
        prompt.push_str(content);
    }

    prompt
}
