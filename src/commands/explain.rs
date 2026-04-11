use anyhow::Result;

use crate::ai::AiEngine;
use crate::config::Config;
use crate::context::ProjectContext;
use crate::output;
use crate::storage::Storage;

pub async fn run(error_message: &[String], json_output: bool) -> Result<()> {
    let error_text = error_message.join(" ");

    if error_text.trim().is_empty() {
        output::print_error("Please provide an error message to explain.");
        return Ok(());
    }

    let config = Config::load()?;
    let context = ProjectContext::detect();

    if !json_output {
        output::print_context_info(&context.project_types);
    }

    let engine = AiEngine::new(config)?;
    let response = engine.analyze(&error_text, Some(&context)).await?;

    // Store in history
    if let Ok(storage) = Storage::open() {
        let fixes_json = serde_json::to_string(&response.fixes).unwrap_or_default();
        let _ = storage.insert(
            "sherlog explain",
            &error_text,
            &response.root_cause,
            &fixes_json,
            &context.summary(),
        );
    }

    output::print_analysis(&response, json_output);
    Ok(())
}
