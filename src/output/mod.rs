use crate::ai::AiResponse;
use colored::Colorize;

/// Output mode for formatting results.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    /// Rich colored terminal output (default).
    Terminal,
    /// Machine-readable JSON.
    Json,
    /// CI/CD format with GitHub Actions annotations.
    Ci,
}

impl OutputMode {
    pub fn from_flags(json: bool, ci: bool) -> Self {
        if ci {
            Self::Ci
        } else if json {
            Self::Json
        } else {
            Self::Terminal
        }
    }
}

pub fn print_analysis(response: &AiResponse, mode: OutputMode) {
    match mode {
        OutputMode::Json => {
            if let Ok(json) = serde_json::to_string_pretty(response) {
                println!("{json}");
            }
        }
        OutputMode::Ci => {
            print_ci_analysis(response);
        }
        OutputMode::Terminal => {
            print_terminal_analysis(response);
        }
    }
}

fn print_ci_analysis(response: &AiResponse) {
    // Root cause as error annotation
    println!(
        "::error title=Root Cause::{}",
        escape_ci(&response.root_cause)
    );

    // Language/tool as notice
    if !response.language_or_tool.is_empty() {
        println!(
            "::notice title=Detected::{}",
            escape_ci(&response.language_or_tool)
        );
    }

    // Explanation
    println!(
        "::error title=Explanation::{}",
        escape_ci(&response.explanation)
    );

    // Fixes as warnings with commands
    for (i, fix) in response.fixes.iter().enumerate() {
        let cmd_part = if fix.command.is_empty() {
            String::new()
        } else {
            format!(" | Command: {}", fix.command)
        };
        println!(
            "::warning title=Fix {} ({}% confidence)::{}{}",
            i + 1,
            fix.confidence,
            escape_ci(&fix.description),
            escape_ci(&cmd_part)
        );
    }

    // Prevention tips as notices
    for tip in &response.prevention_tips {
        println!("::notice title=Prevention::{}", escape_ci(tip));
    }

    // Also output structured JSON for downstream parsing
    println!("::group::Sherlog Analysis JSON");
    if let Ok(json) = serde_json::to_string_pretty(response) {
        println!("{json}");
    }
    println!("::endgroup::");
}

fn escape_ci(s: &str) -> String {
    s.replace('%', "%25")
        .replace('\n', "%0A")
        .replace('\r', "%0D")
}

fn print_terminal_analysis(response: &AiResponse) {
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
