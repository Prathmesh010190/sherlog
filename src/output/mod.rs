use crate::ai::AiResponse;
use colored::Colorize;

pub fn print_analysis(response: &AiResponse, json_output: bool) {
    if json_output {
        if let Ok(json) = serde_json::to_string_pretty(response) {
            println!("{json}");
        }
        return;
    }

    println!();

    // Root cause
    println!(
        "{} {}",
        "\u{274c} Root Cause:".red().bold(),
        response.root_cause.white()
    );
    println!();

    // Language/tool
    if !response.language_or_tool.is_empty() {
        println!(
            "{} {}",
            "\u{1f50d} Detected:".cyan().bold(),
            response.language_or_tool.white()
        );
        println!();
    }

    // Explanation
    println!("{}", "\u{1f4a1} Explanation:".yellow().bold());
    println!("  {}", response.explanation.white());
    println!();

    // Fixes
    if !response.fixes.is_empty() {
        println!(
            "{}",
            "\u{1f527} Fixes (ranked by likelihood):".green().bold()
        );
        for (i, fix) in response.fixes.iter().enumerate() {
            let confidence = format!("{}%", fix.confidence);
            println!(
                "  {}. {} {}",
                (i + 1).to_string().green().bold(),
                fix.description.white(),
                format!("[{confidence}]").dimmed()
            );
            if !fix.command.is_empty() {
                println!("     {} {}", "\u{25b6}".green(), fix.command.cyan());
            }
        }
        println!();
    }

    // Prevention tips
    if !response.prevention_tips.is_empty() {
        println!("{}", "\u{1f6e1}\u{fe0f}  Prevention Tips:".magenta().bold());
        for tip in &response.prevention_tips {
            println!("  {} {}", "\u{2022}".magenta(), tip.white());
        }
        println!();
    }
}

pub fn print_context_info(project_types: &[String]) {
    if !project_types.is_empty() {
        println!(
            "{} {}",
            "\u{1f50d} Detected project context:".cyan().bold(),
            project_types.join(", ").white()
        );
        println!();
    }
}

pub fn print_error(msg: &str) {
    eprintln!("{} {}", "\u{274c} Error:".red().bold(), msg);
}

pub fn print_info(msg: &str) {
    println!("{} {}", "\u{2139}\u{fe0f}".blue(), msg);
}

pub fn print_success(msg: &str) {
    println!("{} {}", "\u{2705}".green(), msg);
}
