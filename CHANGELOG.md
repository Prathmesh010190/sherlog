# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/), and this project adheres to [Semantic Versioning](https://semver.org/).

## [0.2.0] - 2026-04-11

### Added

- `sherlog postmortem --since "2h ago"` command — generates markdown incident postmortem reports from recent error history
- `sherlog patterns --last 7d` command — analyzes error history to identify recurring patterns, most common root causes, failing commands, and daily trends
- `sherlog watch <path>` command — tails a log file in real-time and auto-analyzes errors as they appear
- `--ci` global flag — outputs results in GitHub Actions annotation format (`::error::`, `::warning::`, `::notice::`) for CI/CD pipeline integration
- `install.sh` script — cross-platform installer for Linux and macOS (downloads binary or builds from source)
- `CONTRIBUTING.md` with development setup and coding guidelines
- This `CHANGELOG.md`

### Changed

- All commands now use `OutputMode` enum (Terminal/Json/Ci) instead of boolean flags
- Expanded README with feature comparison table, architecture overview, full command reference, and CI/CD integration guide
- Storage module now supports time-range queries (`get_since`, `get_daily_counts`, `get_root_cause_counts`, `get_command_counts`)
- AI module now supports `generate_text` for free-form responses (used by postmortem)

## [0.1.0] - 2026-04-11

### Added

- Initial release
- `sherlog` — analyze last failed command from shell hook
- `sherlog init` — install shell hook for bash/zsh/fish
- `sherlog analyze <file>` — analyze log files up to 50MB
- `sherlog explain <message>` — explain an error message directly
- `sherlog history` — view past analyses from SQLite database
- `sherlog config` — manage API key, model, and settings
- `sherlog pipe` — read from stdin for piped log analysis
- 7 log parsers: plain text, JSON, syslog, Kubernetes, Docker, Terraform, GitHub Actions
- Project context detection for Node.js, Rust, Python, Go, Docker, Terraform, K8s, GitHub Actions
- Claude API integration with streaming support
- Colored terminal output with `--json` and `--no-color` flags
- SQLite storage at `~/.sherlog/history.db`
- TOML configuration at `~/.sherlog/config.toml`
- Shell hooks for bash, zsh, and fish
- CI workflow for build, test, clippy, and format checks
