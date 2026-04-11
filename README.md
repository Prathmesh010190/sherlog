<p align="center">
  <h1 align="center">sherlog</h1>
  <p align="center">
    <strong>AI-powered error debugging assistant for the terminal</strong>
  </p>
  <p align="center">
    <a href="https://github.com/sherlog-cli/sherlog/actions"><img src="https://github.com/sherlog-cli/sherlog/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
    <a href="https://crates.io/crates/sherlog"><img src="https://img.shields.io/crates/v/sherlog.svg" alt="crates.io"></a>
    <a href="https://github.com/sherlog-cli/sherlog/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT"></a>
  </p>
</p>

Sherlog captures failed commands, analyzes log files, and explains errors using AI — right from your command line. It detects your project context, parses 7+ log formats, and delivers root cause analysis with ranked fixes in seconds.

## Why Sherlog?

Every developer spends hours debugging errors. You copy an error message, search Stack Overflow, read through GitHub issues, and piece together a fix. Sherlog cuts that loop to seconds.

| | Sherlog | Sentry / Datadog | Manual Debugging |
|---|---|---|---|
| **Setup time** | 30 seconds | Hours to days | N/A |
| **Works offline** | Partial (needs API) | No (SaaS) | Yes |
| **Cost** | API usage only | $26+/mo per seat | Free (your time) |
| **Scope** | CLI errors + logs | Production monitoring | Everything |
| **Context-aware** | Auto-detects project | Requires SDK integration | Manual |
| **CI/CD integration** | Built-in `--ci` flag | Requires setup | Custom scripts |
| **Privacy** | Your API key, your data | Third-party SaaS | Fully local |
| **Postmortem reports** | One command | Manual | Manual |

Sherlog isn't a replacement for production monitoring — it's the missing tool for **development-time debugging** that slots in between "read the error" and "open a browser tab."

## Features

- **Automatic error capture** — Shell hook catches failed commands and their stderr automatically
- **AI-powered analysis** — Root cause analysis, ranked fixes, and prevention tips powered by Claude
- **10 CLI commands** — `analyze`, `explain`, `pipe`, `watch`, `postmortem`, `patterns`, and more
- **7 log parsers** — JSON, syslog, plain text stack traces, Kubernetes, Docker, Terraform, GitHub Actions
- **Context-aware** — Auto-detects Node.js, Rust, Python, Go, Docker, Terraform, K8s, and more
- **Real-time watching** — `sherlog watch` tails log files and auto-analyzes errors as they appear
- **Incident postmortems** — Generate markdown postmortem reports from recent error history
- **Pattern analysis** — Identify recurring errors and trends over time
- **CI/CD ready** — `--ci` flag outputs GitHub Actions annotations (`::error::`, `::warning::`)
- **Local history** — All analyses stored in SQLite for later reference
- **Pipe-friendly** — Works with stdin: `kubectl logs pod | sherlog pipe`

## Quick Start

### Install

```bash
# From source (requires Rust)
cargo install --path .

# Or use the install script
curl -fsSL https://raw.githubusercontent.com/sherlog-cli/sherlog/main/install.sh | bash
```

### Configure

```bash
# Set your Anthropic API key
sherlog config set api_key sk-ant-...

# Or use an environment variable
export SHERLOG_API_KEY=sk-ant-...
```

### First Use

```bash
# Install the shell hook (captures failed commands automatically)
sherlog init

# Now just run a command that fails...
cargo build  # (with errors)

# ...and Sherlog has it. Just run:
sherlog
```

## Command Reference

### `sherlog` (no subcommand)

Analyzes the last failed command captured by the shell hook.

```bash
sherlog
sherlog --json
sherlog --ci
```

### `sherlog init`

Installs a shell hook that captures failed commands and their stderr output.

```bash
sherlog init                    # Auto-detect shell
sherlog init --shell bash       # Explicit shell
sherlog init --shell zsh
sherlog init --shell fish
```

Add the output to your shell config file (`~/.bashrc`, `~/.zshrc`, etc.) and restart your shell.

### `sherlog explain <message>`

Explain an error message directly without needing the shell hook.

```bash
sherlog explain "error[E0382]: borrow of moved value"
sherlog explain "ECONNREFUSED 127.0.0.1:5432"
sherlog explain "OOMKilled" --json
```

### `sherlog analyze <file>`

Analyze a log file (up to 50MB). Auto-detects format.

```bash
sherlog analyze /var/log/app/error.log
sherlog analyze build.log --max-lines 500
sherlog analyze output.log --ci
```

### `sherlog pipe`

Read from stdin. Combine with any tool that outputs logs.

```bash
kubectl logs my-pod | sherlog pipe
docker logs my-container 2>&1 | sherlog pipe
journalctl -u myservice --no-pager | sherlog pipe
terraform plan 2>&1 | sherlog pipe
```

### `sherlog watch <path>`

Tail a log file in real-time. Errors are automatically detected and analyzed inline.

```bash
sherlog watch /var/log/app/error.log
sherlog watch ./logs/development.log
```

### `sherlog postmortem`

Generate an incident postmortem report from recent error history.

```bash
sherlog postmortem --since "2h ago"
sherlog postmortem --since "1d ago"
sherlog postmortem --since "30m ago" --json
```

### `sherlog patterns`

Analyze error history to identify recurring patterns and trends.

```bash
sherlog patterns --last 7d
sherlog patterns --last 24h
sherlog patterns --last 30d --json
```

### `sherlog history`

Show past error analyses from the local SQLite database.

```bash
sherlog history
sherlog history --last 5
sherlog history --json
```

### `sherlog config`

Manage configuration.

```bash
sherlog config show
sherlog config set api_key <key>
sherlog config set model claude-sonnet-4-20250514
sherlog config set max_log_lines 500
sherlog config set api_base_url https://my-proxy.example.com
```

## Global Flags

| Flag | Description |
|---|---|
| `--json` | Output results as machine-readable JSON |
| `--ci` | Output in CI/CD format (GitHub Actions annotations) |
| `--no-color` | Disable colored terminal output |

## CI/CD Integration

Use `--ci` to get output formatted as GitHub Actions annotations:

```yaml
# .github/workflows/build.yml
- name: Build
  run: cargo build 2>&1 | tee build.log || true

- name: Analyze build errors
  if: failure()
  run: sherlog analyze build.log --ci
  env:
    SHERLOG_API_KEY: ${{ secrets.SHERLOG_API_KEY }}
```

This produces output like:

```
::error title=Root Cause::Missing dependency 'libssl-dev'
::warning title=Fix 1 (95% confidence)::Install the missing system dependency
```

## Supported Log Formats

| Format | Detection |
|---|---|
| **Plain text stack traces** | Python, Node.js, Rust, Java, Go |
| **JSON structured logs** | Any JSON-per-line format with common field names |
| **Syslog** | Standard syslog format (`Jan 15 10:30:00 host service:`) |
| **Kubernetes** | `kubectl logs`, event output, pod logs |
| **Docker** | `docker logs`, build output, compose logs |
| **Terraform** | Plan/apply errors with HCL file references |
| **GitHub Actions** | `##[error]`, `::error::`, timestamped workflow logs |

## Configuration

Config file location: `~/.sherlog/config.toml`

```toml
api_key = "sk-ant-..."
model = "claude-sonnet-4-20250514"
max_log_lines = 200
theme = "dark"
api_base_url = "https://api.anthropic.com"
```

| Key | Default | Description |
|---|---|---|
| `api_key` | — | Your Anthropic API key |
| `model` | `claude-sonnet-4-20250514` | Claude model to use |
| `max_log_lines` | `200` | Max lines to send for analysis |
| `theme` | `dark` | Terminal theme |
| `api_base_url` | `https://api.anthropic.com` | API endpoint (for proxies) |

The `SHERLOG_API_KEY` environment variable takes precedence over the config file.

## Architecture

```
sherlog/
├── src/
│   ├── main.rs              # Entry point, tokio runtime
│   ├── cli.rs               # Clap CLI definition (10 commands)
│   ├── commands/             # Command implementations
│   │   ├── default.rs        # Auto-analyze last failed command
│   │   ├── init.rs           # Shell hook installer
│   │   ├── analyze.rs        # Log file analysis
│   │   ├── explain.rs        # Direct error explanation
│   │   ├── pipe.rs           # Stdin pipe handler
│   │   ├── watch.rs          # Real-time file watcher
│   │   ├── postmortem.rs     # Incident report generator
│   │   ├── patterns.rs       # Error trend analyzer
│   │   ├── history.rs        # History viewer
│   │   └── config.rs         # Config manager
│   ├── ai/                   # Claude API client
│   ├── parsers/              # 7 log format parsers
│   ├── context/              # Project type detection
│   ├── config/               # TOML config management
│   ├── storage/              # SQLite history database
│   └── output/               # Terminal/JSON/CI formatters
├── hooks/                    # Shell hook scripts
├── tests/                    # Integration tests
└── install.sh                # Cross-platform installer
```

**Key design decisions:**

- **Modular parsers** — Each log format is a separate struct implementing `LogParser`. Auto-detection tries each parser in priority order.
- **Output modes** — All commands support three output modes (terminal, JSON, CI) via a single `OutputMode` enum.
- **Local-first storage** — SQLite at `~/.sherlog/history.db` stores every analysis. No external services beyond the AI API.
- **Async runtime** — Tokio powers HTTP requests and the `watch` command's file monitoring.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup and guidelines.

## License

MIT
