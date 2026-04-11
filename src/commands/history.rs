use anyhow::Result;
use colored::Colorize;

use crate::output;
use crate::storage::Storage;

pub fn run(last: usize, json_output: bool) -> Result<()> {
    let storage = Storage::open()?;
    let entries = storage.get_recent(last)?;

    if entries.is_empty() {
        if json_output {
            println!("[]");
        } else {
            output::print_info("No history entries found. Run sherlog to analyze an error first.");
        }
        return Ok(());
    }

    if json_output {
        let json = serde_json::to_string_pretty(&entries)?;
        println!("{json}");
        return Ok(());
    }

    println!(
        "{}",
        format!("Showing last {} entries:\n", entries.len())
            .cyan()
            .bold()
    );

    for entry in &entries {
        let status = if entry.resolved {
            "\u{2705}".to_string()
        } else {
            "\u{274c}".to_string()
        };

        println!(
            "{} {} #{} — {}",
            status,
            entry.timestamp.dimmed(),
            entry.id.to_string().yellow(),
            entry.command.white().bold()
        );

        if !entry.root_cause.is_empty() {
            println!("  {} {}", "Root cause:".red(), entry.root_cause);
        }

        // Show first fix
        if let Ok(fixes) = serde_json::from_str::<Vec<serde_json::Value>>(&entry.fixes_json) {
            if let Some(fix) = fixes.first() {
                if let Some(desc) = fix.get("description").and_then(|d| d.as_str()) {
                    println!("  {} {}", "Top fix:".green(), desc);
                }
            }
        }

        println!();
    }

    Ok(())
}
