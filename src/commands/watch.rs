use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::Path;
use std::sync::mpsc;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use colored::Colorize;
use notify::{EventKind, RecursiveMode, Watcher};
use regex::Regex;

use crate::ai::AiEngine;
use crate::config::Config;
use crate::context::ProjectContext;
use crate::output::{self, OutputMode};
use crate::parsers;
use crate::storage::Storage;

const ERROR_PATTERN: &str = r"(?i)\b(error|fatal|panic|exception|fail(ed|ure)?|critical|crash)\b";

pub async fn run(path_str: &str, mode: OutputMode) -> Result<()> {
    let path = Path::new(path_str);
    if !path.exists() {
        bail!("File not found: {path_str}");
    }

    let config = Config::load()?;
    let context = ProjectContext::detect();
    let error_re = Regex::new(ERROR_PATTERN)?;

    if mode == OutputMode::Terminal {
        output::print_info(&format!(
            "Watching {path_str} for errors... (Ctrl+C to stop)"
        ));
        output::print_context_info(&context.project_types);
    }

    // Seek to the end of the file so we only watch new content
    let mut file = File::open(path).context("Failed to open file")?;
    file.seek(SeekFrom::End(0))?;
    let mut reader = BufReader::new(file);
    let mut line_buffer = Vec::new();

    // Set up file watcher
    let (tx, rx) = mpsc::channel();
    let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        if let Ok(event) = res {
            if matches!(event.kind, EventKind::Modify(_)) {
                let _ = tx.send(());
            }
        }
    })?;
    watcher.watch(path, RecursiveMode::NonRecursive)?;

    loop {
        // Wait for file modification or timeout for Ctrl+C check
        let _ = rx.recv_timeout(Duration::from_secs(1));

        // Read new lines
        let mut line = String::new();
        while reader.read_line(&mut line)? > 0 {
            let trimmed = line.trim_end().to_string();
            if !trimmed.is_empty() {
                // Check if this line contains an error
                if error_re.is_match(&trimmed) {
                    line_buffer.push(trimmed.clone());

                    // Collect a few more lines for context (stack traces etc.)
                    // Wait briefly for follow-up lines
                    tokio::time::sleep(Duration::from_millis(200)).await;
                    let mut context_line = String::new();
                    while reader.read_line(&mut context_line)? > 0 {
                        let ctx_trimmed = context_line.trim_end().to_string();
                        if !ctx_trimmed.is_empty() {
                            line_buffer.push(ctx_trimmed);
                        }
                        context_line.clear();
                        if line_buffer.len() >= 20 {
                            break;
                        }
                    }

                    // Analyze the batch
                    let error_block = line_buffer.join("\n");
                    line_buffer.clear();

                    if mode == OutputMode::Terminal {
                        println!("{}", "\u{2500}".repeat(60).dimmed());
                        println!(
                            "{} {}",
                            "\u{26a0}\u{fe0f} Error detected:".red().bold(),
                            chrono::Local::now().format("%H:%M:%S").to_string().dimmed()
                        );
                        // Show the raw error lines
                        for eline in error_block.lines().take(5) {
                            println!("  {}", eline.red());
                        }
                        println!();
                    }

                    analyze_error_block(&error_block, &config, &context, mode).await;
                } else {
                    // Non-error line — if we're accumulating context, add it
                    if !line_buffer.is_empty() {
                        line_buffer.push(trimmed);
                    }
                }
            }
            line.clear();
        }
    }
}

async fn analyze_error_block(
    error_text: &str,
    config: &Config,
    context: &ProjectContext,
    mode: OutputMode,
) {
    // Auto-detect format
    let (format_name, _) = parsers::auto_parse(error_text);

    let prompt = format!("Log format: {format_name}\n\n{error_text}");

    match AiEngine::new(config.clone()) {
        Ok(engine) => match engine.analyze(&prompt, Some(context)).await {
            Ok(response) => {
                // Store in history
                if let Ok(storage) = Storage::open() {
                    let fixes_json = serde_json::to_string(&response.fixes).unwrap_or_default();
                    let _ = storage.insert(
                        "sherlog watch",
                        &error_text[..error_text.len().min(10000)],
                        &response.root_cause,
                        &fixes_json,
                        &context.summary(),
                    );
                }

                output::print_analysis(&response, mode);
            }
            Err(e) => {
                output::print_error(&format!("Analysis failed: {e}"));
            }
        },
        Err(e) => {
            output::print_error(&format!("Failed to create AI engine: {e}"));
        }
    }
}
