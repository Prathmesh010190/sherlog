# Contributing to Sherlog

Thank you for considering contributing to Sherlog! This document covers development setup, coding guidelines, and the contribution workflow.

## Development Setup

### Prerequisites

- Rust stable (1.70+) — install via [rustup](https://rustup.rs/)
- Git

### Getting started

```bash
git clone https://github.com/sherlog-cli/sherlog.git
cd sherlog

# Build
cargo build

# Run tests
cargo test

# Run clippy linter
cargo clippy -- -D warnings

# Check formatting
cargo fmt --all -- --check

# Run the binary directly
cargo run -- explain "some error message"
```

### Running specific tests

```bash
# Unit tests only
cargo test --bin sherlog

# Integration tests only
cargo test --test cli_tests

# A specific test
cargo test test_python_traceback
```

## Project Structure

```
src/
├── main.rs          # Entry point — routes CLI args to commands
├── cli.rs           # Clap derive definitions for all commands
├── commands/        # One file per command (analyze.rs, explain.rs, etc.)
├── ai/mod.rs        # Anthropic Claude API client
├── parsers/         # Log format parsers (one file per format)
├── context/mod.rs   # Project type and environment detection
├── config/mod.rs    # TOML config file management
├── storage/mod.rs   # SQLite database for history
└── output/mod.rs    # Terminal, JSON, and CI output formatting
```

## Coding Guidelines

### General

- Use `anyhow` for error handling. Never panic — always return helpful error messages.
- Keep functions focused. If a function does two things, split it.
- Write tests for new parsers and any non-trivial logic.
- Run `cargo fmt`, `cargo clippy -- -D warnings`, and `cargo test` before submitting.

### Adding a new command

1. Add the variant to `Commands` in `src/cli.rs`
2. Create `src/commands/yourcommand.rs`
3. Add `pub mod yourcommand;` to `src/commands/mod.rs`
4. Wire it up in the `match` in `src/main.rs`
5. Add integration tests in `tests/cli_tests.rs`
6. All commands should accept `OutputMode` and handle terminal/JSON/CI output

### Adding a new log parser

1. Create `src/parsers/yourformat.rs`
2. Implement the `LogParser` trait (methods: `name`, `can_parse`, `parse`)
3. Add it to the parser list in `src/parsers/mod.rs` `auto_parse()`
4. Write unit tests in the same file (`#[cfg(test)]` module)
5. Order in `auto_parse()` matters — more specific parsers should come before generic ones

### Output modes

All commands must support three output modes via `OutputMode`:

- **Terminal** — Rich colored output with icons (default)
- **Json** — Machine-readable JSON (`--json` flag)
- **Ci** — GitHub Actions annotations (`--ci` flag)

Use `output::print_analysis()` for AI responses. For custom output, match on `mode`.

### Commit messages

- Use imperative mood: "Add feature" not "Added feature"
- Keep the first line under 72 characters
- Reference issue numbers where applicable

## Pull Request Process

1. Fork the repository and create your branch from `main`
2. Ensure all checks pass: `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test`
3. Update documentation if you're adding or changing commands
4. Submit a PR with a clear description of the change

## Reporting Bugs

Open an issue on GitHub with:

- The command you ran
- The error output
- Your OS and shell (`uname -a`, `echo $SHELL`)
- Sherlog version (`sherlog --version`)
