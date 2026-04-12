<p align="center">
  <h1 align="center">sherlog</h1>
  <p align="center">
    <strong>AI-powered error debugging assistant for the terminal</strong>
  </p>
  <p align="center">
    <a href="https://github.com/Prathmesh010190/sherlog/actions"><img src="https://github.com/Prathmesh010190/sherlog/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
    <a href="https://crates.io/crates/sherlog"><img src="https://img.shields.io/crates/v/sherlog.svg" alt="crates.io"></a>
    <a href="https://github.com/Prathmesh010190/sherlog/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT"></a>
  </p>
</p>

Sherlog captures failed commands, analyzes log files, and explains errors using AI — right from your command line. It detects your project context, parses 7+ log formats, and delivers root cause analysis with ranked fixes in seconds.

## Why Sherlog?

Every developer spends hours debugging errors. You copy an error message, search Stack Overflow, read through GitHub issues, and piece together a fix. Sherlog cuts that loop to seconds.

| | Sherlog | Sentry / Datadog | Manual Debugging |
|---|---|---|---|
| **Setup time** | 30 seconds | Hours to days | N/A |
| **Works offline** | Yes (with Ollama) | No (SaaS) | Yes |
| **Cost** | Free (Ollama) or API usage | $26+/mo per seat | Free (your time) |
| **Scope** | CLI errors + logs | Production monitoring | Everything |
| **Context-aware** | Auto-detects project | Requires SDK integration | Manual |
| **CI/CD integration** | Built-in `--ci` flag | Requires setup | Custom scripts |
| **Privacy** | Fully local with Ollama | Third-party SaaS | Fully local |
| **Postmortem reports** | One command | Manual | Manual |

Sherlog isn't a replacement for production monitoring — it's the missing tool for **development-time debugging** that slots in between "read the error" and "open a browser tab."

## Features

- **Automatic error capture** — Shell hook catches failed commands and their stderr automatically
- **AI-powered analysis** — Root cause analysis, ranked fixes, and prevention tips
- **Dual AI backends** — Claude API (cloud) or Ollama (free, fully local)
- **12 CLI commands** — `analyze`, `explain`, `pipe`, `watch`, `postmortem`, `patterns`, `trace`, `plugin`, and more
- **7 log parsers** — JSON, syslog, plain text stack traces, Kubernetes, Docker, Terraform, GitHub Actions
- **WASM plugin system** — Extend with custom parsers and formatters
- **Multi-service tracing** — Correlate errors across microservices by trace/request ID
- **Context-aware** — Auto-detects Node.js, Rust, Python, Go, Docker, Terraform, K8s, and more
- **Real-time watching** — `sherlog watch` tails log files and auto-analyzes errors as they appear
- **Incident postmortems** — Generate markdown postmortem reports from recent error history
- **Pattern analysis** — Identify recurring errors and trends over time
- **CI/CD ready** — `--ci` flag outputs GitHub Actions annotations (`::error::`, `::warning::`)
- **Local history** — All analyses stored in SQLite for later reference
- **Pipe-friendly** — Works with stdin: `kubectl logs pod | sherlog pipe`

## Quick Start

### Prerequisites

- **Rust** (1.70+) — [Install Rust](https://rustup.rs/)
- **Build tools** — `build-essential`, `pkg-config`, `libssl-dev` (Ubuntu/Debian)
- **AI backend** — Either an Anthropic API key OR Ollama installed locally

### Install

```bash
# Install Rust (skip if already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env

# Install build tools (Ubuntu/Debian)
sudo apt install -y build-essential pkg-config libssl-dev

# Clone and build
git clone https://github.com/Prathmesh010190/sherlog.git
cd sherlog
cargo install --path .
```

---

## Setup Option 1: Free & Local with Ollama (No API Key Needed)

This runs entirely on your machine. No internet required after setup. No cost.

### Step 1: Install Ollama

```bash
curl -fsSL https://ollama.com/install.sh | sh
```

### Step 2: Pull a Model

```bash
# Recommended (3B params, good JSON output)
ollama pull llama3.2

# Or smaller/faster (1.1B params, less accurate)
ollama pull tinyllama
```

### Step 3: Start Ollama

```bash
ollama serve
```

Keep this running in a separate terminal (or run `ollama serve &` to background it).

### Step 4: Configure Sherlog

```bash
sherlog config set provider ollama
sherlog config set model llama3.2
```

### Step 5: Test It

```bash
sherlog explain "error: ECONNREFUSED 127.0.0.1:5432"
```

---

## Setup Option 2: Claude API (Cloud)

Uses Anthropic's Claude models. Requires an API key from [console.anthropic.com](https://console.anthropic.com).

### Step 1: Set Your API Key

```bash
sherlog config set api_key sk-ant-your-key-here

# Or use an environment variable
export SHERLOG_API_KEY=sk-ant-your-key-here
```

### Step 2: Test It

```bash
sherlog explain "error: ECONNREFUSED 127.0.0.1:5432"
```

### Optional: Use a Custom Proxy

```bash
sherlog config set api_base_url https://my-company-proxy.example.com
```

---

## Setup Option 3: GitHub Codespace (No Local Install)

If you don't have disk space or want to try sherlog without installing anything locally:

1. Go to any of your GitHub repos
2. Click **Code** > **Codespaces** > **Create codespace on main**
3. In the codespace terminal, run:

```bash
# Install everything
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env
git clone https://github.com/Prathmesh010190/sherlog.git
cd sherlog
cargo install --path .

# Setup Ollama (free, local)
curl -fsSL https://ollama.com/install.sh | sh
ollama serve &
ollama pull llama3.2
sherlog config set provider ollama
sherlog config set model llama3.2

# Test
sherlog explain "error: ECONNREFUSED 127.0.0.1:5432"
```

---

## Running Sherlog in Your Project

### Auto-Capture Mode (Recommended)

Install the shell hook once, then sherlog automatically catches every failed command:

```bash
# Install the hook
eval "$(sherlog init --shell bash)"    # or zsh, fish

# Now work normally — when something fails:
cargo build        # fails with errors
sherlog            # instantly analyzes the failure

npm start          # fails
sherlog            # analyzes it

python app.py      # crashes
sherlog            # analyzes it
```

To make it permanent, add to your `~/.bashrc` (or `~/.zshrc`):

```bash
eval "$(sherlog init --shell bash)"
```

### Pipe Mode

Pipe any command's output directly:

```bash
cargo build 2>&1 | sherlog pipe
python app.py 2>&1 | sherlog pipe
npm test 2>&1 | sherlog pipe
kubectl logs my-pod | sherlog pipe
docker logs my-container 2>&1 | sherlog pipe
terraform plan 2>&1 | sherlog pipe
```

### Analyze Log Files

```bash
sherlog analyze /var/log/app/error.log
sherlog analyze build.log --max-lines 500
```

### Explain Errors Directly

```bash
sherlog explain "error[E0382]: borrow of moved value"
sherlog explain "ECONNREFUSED 127.0.0.1:5432"
sherlog explain "OOMKilled"
```

---

## All Commands

### `sherlog` (no subcommand)

Analyzes the last failed command captured by the shell hook.

```bash
sherlog                    # Analyze last failed command
sherlog --json             # Output as JSON
sherlog --ci               # Output as GitHub Actions annotations
```

### `sherlog init`

Install a shell hook that captures failed commands and their stderr output.

```bash
sherlog init                    # Auto-detect shell
sherlog init --shell bash       # Bash hook
sherlog init --shell zsh        # Zsh hook
sherlog init --shell fish       # Fish hook
```

Add the output to your shell config (`~/.bashrc`, `~/.zshrc`, `~/.config/fish/config.fish`) and restart your shell.

### `sherlog explain <message>`

Explain an error message directly without needing the shell hook.

```bash
sherlog explain "error[E0382]: borrow of moved value"
sherlog explain "ECONNREFUSED 127.0.0.1:5432"
sherlog explain "OOMKilled" --json
sherlog explain "ModuleNotFoundError: No module named flask" --ci
```

### `sherlog analyze <file>`

Analyze a log file (up to 50MB). Auto-detects format.

```bash
sherlog analyze /var/log/app/error.log
sherlog analyze build.log --max-lines 500
sherlog analyze output.log --ci
sherlog analyze crash.log --json
```

### `sherlog pipe`

Read from stdin. Combine with any tool that outputs logs.

```bash
kubectl logs my-pod | sherlog pipe
docker logs my-container 2>&1 | sherlog pipe
journalctl -u myservice --no-pager | sherlog pipe
terraform plan 2>&1 | sherlog pipe
cargo build 2>&1 | sherlog pipe
npm test 2>&1 | sherlog pipe
python app.py 2>&1 | sherlog pipe
```

### `sherlog watch <path>`

Tail a log file in real-time. Errors are automatically detected and analyzed inline.

```bash
sherlog watch /var/log/app/error.log
sherlog watch ./logs/development.log
```

Press `Ctrl+C` to stop watching.

### `sherlog trace`

Trace errors across multiple services and correlate by request/trace ID.

```bash
# Read from log files (one per service)
sherlog trace --services api,worker,db api.log worker.log db.log

# Auto-read from Docker containers
sherlog trace --services api,worker,db

# Auto-read from Kubernetes pods
sherlog trace --services api,worker,db

# Custom time window
sherlog trace --services api,worker,db --since "30m ago"

# JSON output
sherlog trace --services api,worker --json
```

### `sherlog postmortem`

Generate an incident postmortem report from recent error history.

```bash
sherlog postmortem                      # Default: last 2 hours
sherlog postmortem --since "2h ago"
sherlog postmortem --since "1d ago"
sherlog postmortem --since "30m ago" --json
```

### `sherlog patterns`

Analyze error history to identify recurring patterns and trends.

```bash
sherlog patterns                        # Default: last 7 days
sherlog patterns --last 7d
sherlog patterns --last 24h
sherlog patterns --last 30d --json
```

### `sherlog history`

Show past error analyses from the local SQLite database.

```bash
sherlog history                         # Default: last 10 entries
sherlog history --last 5
sherlog history --last 20
sherlog history --json
```

### `sherlog plugin`

Manage WASM plugins for custom log parsers and output formatters.

```bash
sherlog plugin list                     # List installed plugins
sherlog plugin install <url>            # Install from URL
sherlog plugin remove <name>            # Remove by name
```

### `sherlog config`

Manage configuration.

```bash
sherlog config show                     # Show all settings

# Set values
sherlog config set api_key <key>        # Anthropic API key
sherlog config set provider ollama      # Switch to Ollama (local)
sherlog config set provider anthropic   # Switch to Claude API
sherlog config set model llama3.2       # Change AI model
sherlog config set model claude-sonnet-4-20250514
sherlog config set max_log_lines 500    # Max lines to analyze
sherlog config set api_base_url https://my-proxy.example.com
sherlog config set telemetry true       # Enable anonymous telemetry
sherlog config set telemetry false      # Disable telemetry
sherlog config set theme dark           # Terminal theme
```

## Global Flags

Every command supports these flags:

| Flag | Description |
|---|---|
| `--json` | Output results as machine-readable JSON |
| `--ci` | Output in CI/CD format (GitHub Actions `::error::` annotations) |
| `--no-color` | Disable colored terminal output |
| `-h, --help` | Print help for the command |
| `-V, --version` | Print sherlog version |

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
| **WASM plugins** | Any custom format via installed plugins |

## Configuration

Config file location: `~/.sherlog/config.toml`

```toml
api_key = "sk-ant-..."
model = "claude-sonnet-4-20250514"
max_log_lines = 200
theme = "dark"
api_base_url = "https://api.anthropic.com"
provider = "anthropic"
telemetry = false
```

| Key | Default | Description |
|---|---|---|
| `api_key` | — | Your Anthropic API key (not needed for Ollama) |
| `model` | `claude-sonnet-4-20250514` | AI model to use |
| `max_log_lines` | `200` | Max lines to send for analysis |
| `theme` | `dark` | Terminal theme |
| `api_base_url` | `https://api.anthropic.com` | API endpoint (auto-changes with provider) |
| `provider` | `anthropic` | AI provider: `anthropic` or `ollama` |
| `telemetry` | `false` | Enable anonymous usage stats (opt-in) |

The `SHERLOG_API_KEY` environment variable takes precedence over the config file.

## Architecture

```
sherlog/
├── src/
│   ├── main.rs              # Entry point, tokio runtime
│   ├── cli.rs               # Clap CLI definition (12 commands)
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
│   │   ├── config.rs         # Config manager
│   │   ├── trace.rs          # Multi-service correlation
│   │   └── plugin.rs         # Plugin management
│   ├── ai/                   # Claude + Ollama API clients
│   ├── parsers/              # 7 log format parsers + plugin integration
│   ├── plugins/              # WASM plugin system (Extism)
│   ├── context/              # Project type detection
│   ├── config/               # TOML config management
│   ├── storage/              # SQLite history database
│   ├── telemetry/            # Opt-in usage tracking
│   └── output/               # Terminal/JSON/CI formatters + plugin formatters
├── plugins/examples/         # Example WASM plugin (CSV parser)
├── hooks/                    # Shell hook scripts
├── tests/                    # Integration tests
└── install.sh                # Cross-platform installer
```

**Key design decisions:**

- **Modular parsers** — Each log format is a separate struct implementing `LogParser`. Auto-detection tries WASM plugins first, then built-in parsers in priority order.
- **Dual AI backends** — Anthropic Claude (cloud) and Ollama (local) share the same interface. Switch with one config command.
- **Output modes** — All commands support three output modes (terminal, JSON, CI) via a single `OutputMode` enum, plus WASM plugin formatters.
- **Local-first storage** — SQLite at `~/.sherlog/history.db` stores every analysis. No external services beyond the AI API.
- **Async runtime** — Tokio powers HTTP requests and the `watch` command's file monitoring.
- **WASM plugins** — Extism-based plugin system allows custom parsers and formatters without recompiling sherlog.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup and guidelines.

## License

MIT
