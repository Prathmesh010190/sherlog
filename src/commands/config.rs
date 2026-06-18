use anyhow::Result;
use colored::Colorize;

use crate::cli::ConfigAction;
use crate::config::Config;
use crate::output::{self, OutputMode};

pub fn run(action: &ConfigAction, mode: OutputMode) -> Result<()> {
    match action {
        ConfigAction::Set { key, value } => {
            let mut config = Config::load()?;
            config.set_value(key, value)?;
            config.save()?;

            match mode {
                OutputMode::Json | OutputMode::Ci => {
                    println!(r#"{{"status": "ok", "key": "{key}", "value": "{value}"}}"#);
                }
                OutputMode::Terminal => {
                    let display_value = if key == "api_key" {
                        mask_api_key(value)
                    } else {
                        value.to_string()
                    };
                    output::print_success(&format!("Set {key} = {display_value}"));
                }
            }
        }
        ConfigAction::Show => {
            let config = Config::load()?;

            match mode {
                OutputMode::Json | OutputMode::Ci => {
                    let mut display = config.clone();
                    display.api_key = display.api_key.as_ref().map(|k| mask_api_key(k));
                    let json = serde_json::to_string_pretty(&display)?;
                    println!("{json}");
                }
                OutputMode::Terminal => {
                    println!("{}", "Sherlog Configuration".cyan().bold());
                    println!("{}", "\u{2500}".repeat(40).dimmed());

                    let api_key_display = match config.api_key() {
                        Some(key) => mask_api_key(&key),
                        None => "(not set)".dimmed().to_string(),
                    };

                    let api_key_source = if std::env::var("SHERLOG_API_KEY").is_ok() {
                        " (from SHERLOG_API_KEY env)".dimmed().to_string()
                    } else {
                        String::new()
                    };

                    println!(
                        "  {} {}{}",
                        "api_key:".white().bold(),
                        api_key_display,
                        api_key_source
                    );
                    println!("  {} {}", "model:".white().bold(), config.model);
                    println!(
                        "  {} {}",
                        "max_log_lines:".white().bold(),
                        config.max_log_lines
                    );
                    println!("  {} {}", "theme:".white().bold(), config.theme);
                    println!(
                        "  {} {}",
                        "api_base_url:".white().bold(),
                        config.api_base_url
                    );
                    println!("  {} {}", "provider:".white().bold(), config.provider);
                    println!(
                        "  {} {}",
                        "telemetry:".white().bold(),
                        if config.telemetry {
                            "enabled"
                        } else {
                            "disabled"
                        }
                    );

                    println!();
                    println!(
                        "  {} {}",
                        "Config file:".dimmed(),
                        Config::config_path()
                            .map(|p| p.display().to_string())
                            .unwrap_or_else(|_| "unknown".to_string())
                    );
                }
            }
        }
    }

    Ok(())
}

fn mask_api_key(key: &str) -> String {
    if key.len() <= 8 {
        return "*".repeat(key.len());
    }
    let visible = &key[..4];
    format!(
        "{visible}{}...{}",
        "*".repeat(key.len() - 8),
        &key[key.len() - 4..]
    )
}
