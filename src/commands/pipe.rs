use anyhow::{bail, Result};
use std::io::{self, IsTerminal, Read};

use crate::ai::AiEngine;
use crate::config::Config;
use crate::context::ProjectContext;
use crate::output::{self, OutputMode};
use crate::parsers;
use crate::storage::Storage;

pub async fn run(mode: OutputMode) -> Result<()> {
    // Check if stdin is a terminal (no pipe)
    if io::stdin().is_terminal() {
        bail!("No input detected. Pipe data to sherlog:\n  kubectl logs pod | sherlog pipe\n  docker logs container 2>&1 | sherlog pipe");
    }

    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .map_err(|e| anyhow::anyhow!("Failed to read from stdin: {e}"))?;

    if input.trim().is_empty() {
        bail!("Empty input received. Nothing to analyze.");
    }

    // Strip ANSI escape codes
    let stripped_bytes = strip_ansi_escapes::strip(input.as_bytes());
    let input = String::from_utf8_lossy(&stripped_bytes).to_string();

    // Apply line limit
    let config = Config::load()?;
    let lines: Vec<&str> = input.lines().collect();
    let truncated = if lines.len() > config.max_log_lines {
        if mode == OutputMode::Terminal {
            output::print_info(&format!(
                "Input has {} lines. Analyzing the last {} lines.",
                lines.len(),
                config.max_log_lines
            ));
        }
        lines[lines.len() - config.max_log_lines..].join("\n")
    } else {
        input.clone()
    };

    // Auto-detect format
    let (format_name, _parsed) = parsers::auto_parse(&truncated);

    if mode == OutputMode::Terminal {
        output::print_info(&format!("Detected format: {format_name}"));
    }

    let context = ProjectContext::detect();
    if mode == OutputMode::Terminal {
        output::print_context_info(&context.project_types);
    }

    let engine = AiEngine::new(config)?;
    let response = engine.analyze(&truncated, Some(&context)).await?;

    // Store in history
    if let Ok(storage) = Storage::open() {
        let fixes_json = serde_json::to_string(&response.fixes).unwrap_or_default();
        let _ = storage.insert(
            "sherlog pipe",
            &truncated[..truncated.len().min(10000)],
            &response.root_cause,
            &fixes_json,
            &context.summary(),
        );
    }

    output::print_analysis(&response, mode);
    Ok(())
}
