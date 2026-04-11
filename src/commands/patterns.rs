use anyhow::Result;
use colored::Colorize;

use crate::commands::postmortem::parse_since;
use crate::output::{self, OutputMode};
use crate::storage::Storage;

pub fn run(last: &str, mode: OutputMode) -> Result<()> {
    let since_datetime = parse_since(last)?;

    let storage = Storage::open()?;
    let total = storage.get_count_since(&since_datetime)?;

    if total == 0 {
        if mode == OutputMode::Json {
            println!(r#"{{"error": "No errors found in the specified time window"}}"#);
        } else {
            output::print_info(&format!(
                "No errors found since {since_datetime}. Nothing to analyze."
            ));
        }
        return Ok(());
    }

    let root_cause_counts = storage.get_root_cause_counts(&since_datetime)?;
    let daily_counts = storage.get_daily_counts(&since_datetime)?;
    let command_counts = storage.get_command_counts(&since_datetime)?;

    match mode {
        OutputMode::Json | OutputMode::Ci => {
            let json = serde_json::json!({
                "since": since_datetime,
                "total_errors": total,
                "top_root_causes": root_cause_counts.iter().map(|(cause, count)| {
                    serde_json::json!({"root_cause": cause, "count": count})
                }).collect::<Vec<_>>(),
                "daily_trend": daily_counts.iter().map(|(day, count)| {
                    serde_json::json!({"date": day, "count": count})
                }).collect::<Vec<_>>(),
                "top_commands": command_counts.iter().map(|(cmd, count)| {
                    serde_json::json!({"command": cmd, "count": count})
                }).collect::<Vec<_>>(),
            });

            if mode == OutputMode::Ci {
                if let Some((top_cause, top_count)) = root_cause_counts.first() {
                    println!("::warning title=Most Common Error ({top_count}x)::{top_cause}");
                }
                println!(
                    "::notice title=Error Patterns::Total: {total} errors since {since_datetime}"
                );
                println!("::group::Sherlog Pattern Analysis JSON");
            }
            println!("{}", serde_json::to_string_pretty(&json)?);
            if mode == OutputMode::Ci {
                println!("::endgroup::");
            }
        }
        OutputMode::Terminal => {
            println!(
                "{}\n",
                format!("Error Pattern Analysis (since {since_datetime})")
                    .cyan()
                    .bold()
                    .underline()
            );

            println!(
                "  {} {}\n",
                "Total errors:".white().bold(),
                total.to_string().yellow()
            );

            // Daily trend
            if !daily_counts.is_empty() {
                println!("{}", "\u{1f4c8} Daily Trend:".green().bold());
                let max_count = daily_counts.iter().map(|(_, c)| *c).max().unwrap_or(1);
                for (day, count) in &daily_counts {
                    let bar_len = (*count as f64 / max_count as f64 * 30.0) as usize;
                    let bar: String = "\u{2588}".repeat(bar_len);
                    println!("  {} {} {}", day.dimmed(), bar.green(), count);
                }
                println!();
            }

            // Top root causes
            if !root_cause_counts.is_empty() {
                println!("{}", "\u{1f534} Most Common Root Causes:".red().bold());
                for (i, (cause, count)) in root_cause_counts.iter().take(10).enumerate() {
                    println!(
                        "  {}. {} {}",
                        (i + 1).to_string().red().bold(),
                        cause.white(),
                        format!("({count}x)").dimmed()
                    );
                }
                println!();
            }

            // Top commands
            if !command_counts.is_empty() {
                println!("{}", "\u{1f527} Most Failing Commands:".yellow().bold());
                for (cmd, count) in &command_counts {
                    println!("  {} {}", cmd.cyan(), format!("({count}x)").dimmed());
                }
                println!();
            }
        }
    }

    Ok(())
}
