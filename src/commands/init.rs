use std::io::IsTerminal;

use anyhow::{bail, Result};

use crate::output;

const BASH_HOOK: &str = r#"# Sherlog shell hook for Bash
# Captures failed commands and their stderr for analysis

__sherlog_preexec() {
    export __SHERLOG_LAST_CMD="$1"
}

__sherlog_precmd() {
    local exit_code=$?
    if [ $exit_code -ne 0 ] && [ -n "$__SHERLOG_LAST_CMD" ]; then
        cat <<SHERLOG_JSON > /tmp/sherlog_last_error.json
{
  "command": "$(echo "$__SHERLOG_LAST_CMD" | sed 's/"/\\"/g')",
  "exit_code": $exit_code,
  "stderr": "",
  "working_directory": "$(pwd)",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
SHERLOG_JSON
    fi
    unset __SHERLOG_LAST_CMD
}

trap '__sherlog_preexec "$BASH_COMMAND"' DEBUG
PROMPT_COMMAND="__sherlog_precmd;${PROMPT_COMMAND:+$PROMPT_COMMAND}"
"#;

const ZSH_HOOK: &str = r#"# Sherlog shell hook for Zsh
# Captures failed commands and their stderr for analysis

__sherlog_preexec() {
    export __SHERLOG_LAST_CMD="$1"
}

__sherlog_precmd() {
    local exit_code=$?
    if [[ $exit_code -ne 0 ]] && [[ -n "$__SHERLOG_LAST_CMD" ]]; then
        cat <<SHERLOG_JSON > /tmp/sherlog_last_error.json
{
  "command": "$(echo "$__SHERLOG_LAST_CMD" | sed 's/"/\\"/g')",
  "exit_code": $exit_code,
  "stderr": "",
  "working_directory": "$(pwd)",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
SHERLOG_JSON
    fi
    unset __SHERLOG_LAST_CMD
}

autoload -Uz add-zsh-hook
add-zsh-hook preexec __sherlog_preexec
add-zsh-hook precmd __sherlog_precmd
"#;

const FISH_HOOK: &str = r#"# Sherlog shell hook for Fish
# Captures failed commands and their stderr for analysis

function __sherlog_on_exit --on-event fish_postexec
    set -l exit_code $status
    set -l cmd $argv[1]

    if test $exit_code -ne 0
        set -l timestamp (date -u +%Y-%m-%dT%H:%M:%SZ)
        set -l working_dir (pwd)
        set -l escaped_cmd (string replace -a '"' '\\"' -- "$cmd")

        echo '{
  "command": "'$escaped_cmd'",
  "exit_code": '$exit_code',
  "stderr": "",
  "working_directory": "'$working_dir'",
  "timestamp": "'$timestamp'"
}' > /tmp/sherlog_last_error.json
    end
end
"#;

pub fn run(shell: Option<String>) -> Result<()> {
    let detected = shell.unwrap_or_else(detect_shell);

    let (hook, config_file, source_cmd) = match detected.as_str() {
        "bash" => (
            BASH_HOOK,
            "~/.bashrc",
            "source <(sherlog init --shell bash)",
        ),
        "zsh" => (ZSH_HOOK, "~/.zshrc", "source <(sherlog init --shell zsh)"),
        "fish" => (
            FISH_HOOK,
            "~/.config/fish/config.fish",
            "sherlog init --shell fish | source",
        ),
        other => bail!(
            "Unsupported shell: '{}'. Supported shells: bash, zsh, fish",
            other
        ),
    };

    // If stdout is being captured (for `source <(...)`), just print the hook
    if std::io::stdout().is_terminal() {
        // Interactive mode — print instructions
        output::print_success(&format!("Shell hook for {detected}:"));
        println!();
        println!("{hook}");
        println!();
        output::print_info(&format!("Add this line to {config_file}:"));
        println!();
        println!("  {source_cmd}");
        println!();
        output::print_info("Then restart your shell or run:");
        println!("  source {config_file}");
    } else {
        // Being captured via source <(...)
        print!("{hook}");
    }

    Ok(())
}

fn detect_shell() -> String {
    if let Ok(shell) = std::env::var("SHELL") {
        if shell.contains("zsh") {
            return "zsh".to_string();
        }
        if shell.contains("fish") {
            return "fish".to_string();
        }
        if shell.contains("bash") {
            return "bash".to_string();
        }
    }
    "bash".to_string()
}
