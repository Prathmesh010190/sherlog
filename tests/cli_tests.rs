use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn sherlog() -> Command {
    Command::cargo_bin("sherlog").unwrap()
}

#[test]
fn test_help_flag() {
    sherlog()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Sherlog captures failed commands"));
}

#[test]
fn test_version_flag() {
    sherlog()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("sherlog"));
}

#[test]
fn test_no_subcommand_without_error_file() {
    // Remove the error file if it exists
    let _ = fs::remove_file("/tmp/sherlog_last_error.json");

    sherlog()
        .assert()
        .success()
        .stderr(predicate::str::contains("sherlog init"));
}

#[test]
fn test_init_bash() {
    sherlog()
        .args(["init", "--shell", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("__sherlog_preexec"));
}

#[test]
fn test_init_zsh() {
    sherlog()
        .args(["init", "--shell", "zsh"])
        .assert()
        .success()
        .stdout(predicate::str::contains("add-zsh-hook"));
}

#[test]
fn test_init_fish() {
    sherlog()
        .args(["init", "--shell", "fish"])
        .assert()
        .success()
        .stdout(predicate::str::contains("fish_postexec"));
}

#[test]
fn test_init_unsupported_shell() {
    sherlog()
        .args(["init", "--shell", "powershell"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unsupported shell"));
}

#[test]
fn test_explain_empty() {
    sherlog()
        .args(["explain", ""])
        .assert()
        .success()
        .stderr(predicate::str::contains("provide an error message"));
}

#[test]
fn test_analyze_missing_file() {
    sherlog()
        .args(["analyze", "/nonexistent/file.log"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("File not found"));
}

#[test]
fn test_config_show() {
    sherlog()
        .args(["config", "show"])
        .assert()
        .success()
        .stdout(predicate::str::contains("model"));
}

#[test]
fn test_config_set_invalid_key() {
    sherlog()
        .args(["config", "set", "nonexistent_key", "value"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unknown config key"));
}

#[test]
fn test_history_empty() {
    sherlog().args(["history"]).assert().success();
}

#[test]
fn test_history_json_output() {
    sherlog().args(["history", "--json"]).assert().success();
}

#[test]
fn test_pipe_no_stdin() {
    sherlog().arg("pipe").assert().failure().stderr(
        predicate::str::contains("Empty input received")
            .or(predicate::str::contains("No input detected")),
    );
}

#[test]
fn test_analyze_file_with_json_output() {
    let dir = TempDir::new().unwrap();
    let log_path = dir.path().join("test.log");
    fs::write(
        &log_path,
        "ERROR: Connection refused\nWARNING: Retrying...\n",
    )
    .unwrap();

    // This will fail because no API key is set, which is expected
    sherlog()
        .args(["analyze", log_path.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("API key"));
}

#[test]
fn test_explain_without_api_key() {
    // Ensure no API key is set
    sherlog()
        .env_remove("SHERLOG_API_KEY")
        .args(["explain", "some error message"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("API key"));
}

#[test]
fn test_no_color_flag() {
    sherlog()
        .args(["--no-color", "config", "show"])
        .assert()
        .success();
}

#[test]
fn test_json_flag() {
    sherlog()
        .args(["--json", "config", "show"])
        .assert()
        .success()
        .stdout(predicate::str::contains("model"));
}

// --- New command tests ---

#[test]
fn test_ci_flag() {
    sherlog()
        .args(["--ci", "config", "show"])
        .assert()
        .success()
        .stdout(predicate::str::contains("model"));
}

#[test]
fn test_ci_flag_with_history() {
    sherlog().args(["--ci", "history"]).assert().success();
}

#[test]
fn test_postmortem_no_errors() {
    sherlog()
        .args(["postmortem", "--since", "1m ago"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("No errors found")
                .or(predicate::str::contains("Nothing to report")),
        );
}

#[test]
fn test_postmortem_json() {
    sherlog()
        .args(["--json", "postmortem", "--since", "1m ago"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No errors found"));
}

#[test]
fn test_postmortem_invalid_since() {
    sherlog()
        .args(["postmortem", "--since", "potato"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid time format"));
}

#[test]
fn test_patterns_no_errors() {
    sherlog()
        .args(["patterns", "--last", "1m"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("No errors found")
                .or(predicate::str::contains("Nothing to analyze")),
        );
}

#[test]
fn test_patterns_json() {
    sherlog()
        .args(["--json", "patterns", "--last", "1m"])
        .assert()
        .success();
}

#[test]
fn test_patterns_invalid_time() {
    sherlog()
        .args(["patterns", "--last", "banana"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid time format"));
}

#[test]
fn test_watch_missing_file() {
    sherlog()
        .args(["watch", "/nonexistent/file.log"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("File not found"));
}

#[test]
fn test_help_shows_new_commands() {
    sherlog().arg("--help").assert().success().stdout(
        predicate::str::contains("postmortem")
            .and(predicate::str::contains("patterns"))
            .and(predicate::str::contains("watch")),
    );
}
