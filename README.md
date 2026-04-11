# sherlog

A developer-focused error debugging assistant for the terminal. Sherlog captures failed commands, analyzes log files, and explains errors using AI — right from your command line.

## Features

- **Automatic error capture** — Install a shell hook that catches failed commands and their stderr output automatically
- **AI-powered analysis** — Get root cause analysis, ranked fixes, and prevention tips powered by Claude
- **Log file analysis** — Parse and analyze log files up to 50MB (JSON, syslog, plain text, Kubernetes, Docker, Terraform, GitHub Actions)
- **Context-aware** — Automatically detects your project type (Node.js, Rust, Python, Go, Docker, Terraform, etc.) for smarter analysis
- **Local history** — All analyses are stored in a local SQLite database for later reference
- **Pipe-friendly** — Works with stdin so you can pipe logs from any source

## Installation

### From source

```bash
cargo install --path .
```

### From crates.io

```bash
cargo install sherlog
```

## Quick Start

1. Set your Anthropic API key:

```bash
sherlog config set api_key sk-ant-...
# or use an environment variable
export SHERLOG_API_KEY=sk-ant-...
```

2. Install the shell hook for automatic error capture:

```bash
sherlog init
```

3. Follow the instructions to add the hook to your shell config, then:

```bash
# Run a command that fails...
cargo build  # (with errors)

# Sherlog automatically captured the error. Just run:
sherlog
```

## Usage

### Analyze the last failed command

```bash
sherlog
```

### Explain an error message directly

```bash
sherlog explain "error[E0382]: borrow of moved value: \`x\`"
```

### Analyze a log file

```bash
sherlog analyze /var/log/app/error.log
```

### Pipe logs from any source

```bash
kubectl logs my-pod | sherlog pipe
docker logs my-container 2>&1 | sherlog pipe
journalctl -u myservice --no-pager | sherlog pipe
```

### View past analyses

```bash
sherlog history
sherlog history --last 5
```

### Configuration

```bash
sherlog config show
sherlog config set api_key <your-key>
sherlog config set model claude-sonnet-4-20250514
```

## Supported Log Formats

- Plain text stack traces (Python, Node.js, Rust, Java, Go)
- JSON structured logs
- Syslog
- Kubernetes event logs
- Docker logs
- Terraform error output
- GitHub Actions logs

## Configuration

Config is stored at `~/.sherlog/config.toml`:

```toml
api_key = "sk-ant-..."
model = "claude-sonnet-4-20250514"
max_log_lines = 200
theme = "dark"
api_base_url = "https://api.anthropic.com"
```

The `SHERLOG_API_KEY` environment variable takes precedence over the config file.

## JSON Output

All commands support `--json` for machine-readable output:

```bash
sherlog explain "segfault" --json
sherlog analyze app.log --json
```

## License

MIT
