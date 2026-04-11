use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;

use crate::ai::AiEngine;
use crate::config::Config;
use crate::context::ProjectContext;
use crate::output;
use crate::storage::Storage;

const LAST_ERROR_PATH: &str = "/tmp/sherlog_last_error.json";

#[derive(Deserialize)]
struct CapturedError {
    command: String,
    exit_code: i32,
    stderr: String,
    working_directory: String,
    timestamp: String,
}

pub async fn run(json_output: bool) -> Result<()> {
    let path = std::path::Path::new(LAST_ERROR_PATH);
    if !path.exists() {
        output::print_error(
            "No captured error found. Install the shell hook first:\n  sherlog init",
        );
        return Ok(());
    }

    let contents = fs::read_to_string(path).context("Failed to read last error file")?;
    let captured: CapturedError =
        serde_json::from_str(&contents).context("Failed to parse captured error")?;

    let config = Config::load()?;
    let context = ProjectContext::detect();

    if !json_output {
        output::print_context_info(&context.project_types);
        output::print_info(&format!(
            "Analyzing failed command: {} (exit code {})",
            captured.command, captured.exit_code
        ));
        output::print_info(&format!("Directory: {}", captured.working_directory));
        output::print_info(&format!("Time: {}", captured.timestamp));
        println!();
    }

    let error_text = if captured.stderr.is_empty() {
        format!(
            "Command `{}` failed with exit code {}",
            captured.command, captured.exit_code
        )
    } else {
        format!(
            "Command `{}` failed with exit code {}.\n\nStderr output:\n{}",
            captured.command, captured.exit_code, captured.stderr
        )
    };

    let engine = AiEngine::new(config)?;
    let response = engine.analyze(&error_text, Some(&context)).await?;

    // Store in history
    if let Ok(storage) = Storage::open() {
        let fixes_json = serde_json::to_string(&response.fixes).unwrap_or_default();
        let _ = storage.insert(
            &captured.command,
            &error_text,
            &response.root_cause,
            &fixes_json,
            &context.summary(),
        );
    }

    output::print_analysis(&response, json_output);

    // Clean up the error file after successful analysis
    let _ = fs::remove_file(path);

    Ok(())
}
