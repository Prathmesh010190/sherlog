<div align="center">

# 🔍 sherlog

**Your terminal's error detective.**
Catch a failed command, get the root cause and ranked fixes. No browser tab needed.

[![CI](https://github.com/Prathmesh010190/sherlog/actions/workflows/ci.yml/badge.svg)](https://github.com/Prathmesh010190/sherlog/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
![Built with Rust](https://img.shields.io/badge/built%20with-Rust-orange?logo=rust)
![Local AI](https://img.shields.io/badge/works%20offline-Ollama-green)

[Quick start](#-quick-start) · [How it works](#-how-it-works) · [CI mode](#-ci-mode) · [Commands](#-commands) · [Roadmap](#-roadmap)

<!-- Replace with your demo: record with `asciinema` or `vhs`, then drop the GIF here -->
<!-- <img src="docs/demo.gif" alt="sherlog demo" width="720"> -->

</div>

---

## Why sherlog?

Every failed build follows the same loop: copy the error, search, skim three threads, try a fix. Sherlog removes the copy-paste.

```bash
cargo build          # fails
sherlog              # reads the failure, detects your project, explains it
```

It looks at what actually failed, works out your project context (Rust, Node, Python, Go, Docker, Terraform, K8s), and returns a **root cause**, **ranked fixes with confidence**, and **prevention tips**.

## ✨ Highlights

| | |
|---|---|
| 🪝 **Auto-capture** | A shell hook records failed commands and their stderr. Just type `sherlog`. |
| 🧠 **Two AI backends** | Claude API in the cloud, or **Ollama fully local**. Your logs never have to leave your machine. |
| 📜 **7 log parsers** | JSON, syslog, stack traces, Kubernetes, Docker, Terraform, GitHub Actions. |
| 🤖 **CI-native** | `--ci` emits GitHub Actions annotations. |
| 🔌 **WASM plugins** | Add custom parsers and formatters without recompiling. |
| 🧵 **Multi-service tracing** | Correlate errors across services by trace/request ID. |
| 🗂️ **Local history** | Every analysis is stored in SQLite for postmortems and pattern analysis. |

## 🚀 Quick start

**1. Install**

```bash
git clone https://github.com/Prathmesh010190/sherlog.git
cd sherlog
cargo install --path .
```

> Needs Rust 1.70+, plus `build-essential pkg-config libssl-dev` on Ubuntu/Debian.

**2. Pick a brain**

<table>
<tr>
<td width="50%" valign="top">

**🏠 Local and free (Ollama)**

```bash
ollama pull llama3.2
sherlog config set provider ollama
sherlog config set model llama3.2
```

</td>
<td width="50%" valign="top">

**☁️ Cloud (Claude API)**

```bash
sherlog config set api_key sk-ant-...
# or
export SHERLOG_API_KEY=sk-ant-...
```

</td>
</tr>
</table>

**3. Try it**

```bash
sherlog explain "error: ECONNREFUSED 127.0.0.1:5432"
```

**4. Turn on auto-capture (optional)**

```bash
eval "$(sherlog init --shell bash)"   # or zsh, fish. Add to your rc file to keep it.
```

## 🧭 How it works

```
 failed command / log file / stdin
              │
              ▼
   ┌─────────────────────┐
   │ Parse (plugins →    │   7 built-in formats + WASM plugins
   │  built-in parsers)  │
   └──────────┬──────────┘
              ▼
   ┌─────────────────────┐
   │ Detect project      │   Rust · Node · Python · Go · Docker · TF · K8s
   └──────────┬──────────┘
              ▼
   ┌─────────────────────┐
   │ Ask AI              │   Claude (cloud)  or  Ollama (local)
   └──────────┬──────────┘
              ▼
   Terminal  ·  JSON  ·  CI annotations      (+ saved to SQLite)
```

## 🤖 CI mode

Sherlog turns a failing build into annotations on the GitHub Actions run page.

```yaml
- name: Build
  run: cargo build 2>&1 | tee build.log || true

- name: Explain failures
  if: failure()
  run: sherlog analyze build.log --ci
  env:
    SHERLOG_API_KEY: ${{ secrets.SHERLOG_API_KEY }}
```

Output:

```
::error title=Root Cause::Missing dependency 'libssl-dev'
::warning title=Fix 1 (95% confidence)::Install the missing system dependency
```

## ⚖️ When to use sherlog

| Use sherlog when | Use something else when |
|---|---|
| You want errors explained **in the terminal**, with project context | You need production monitoring and alerting (Sentry, Datadog) |
| Logs are sensitive and must stay **local** (Ollama) | You want a full AI coding agent that edits files (Claude Code, Aider) |
| You want a **CI step** that explains failures | You live in an IDE assistant that already sees your errors |

Sherlog is a focused debugging tool, not a replacement for any of those.

## 🧰 Commands

Every command supports `--json`, `--ci`, `--no-color`, `-h`, and `-V`.

| Command | What it does |
|---|---|
| `sherlog` | Analyze the last failed command (needs the shell hook) |
| `sherlog explain "<msg>"` | Explain an error message directly |
| `sherlog analyze <file>` | Analyze a log file (up to 50 MB, `--max-lines N`) |
| `sherlog pipe` | Read from stdin: `kubectl logs pod \| sherlog pipe` |
| `sherlog watch <path>` | Tail a log file and analyze errors live |
| `sherlog trace` | Correlate errors across services by trace ID |
| `sherlog postmortem` | Generate an incident report from recent history |
| `sherlog patterns` | Find recurring errors and trends |
| `sherlog history` | Browse past analyses |
| `sherlog plugin` | `list`, `install <url>`, `remove <name>` |
| `sherlog init` | Print the shell hook for bash, zsh, or fish |
| `sherlog config` | `show` / `set <key> <value>` |

<details>
<summary><b>Usage examples</b></summary>

```bash
# Pipe anything
cargo build 2>&1 | sherlog pipe
docker logs my-container 2>&1 | sherlog pipe
terraform plan 2>&1 | sherlog pipe
journalctl -u myservice --no-pager | sherlog pipe

# Trace across services
sherlog trace --services api,worker,db api.log worker.log db.log
sherlog trace --services api,worker --since "30m ago" --json

# Reports
sherlog postmortem --since "2h ago"
sherlog patterns --last 7d
sherlog history --last 20
```

</details>

<details>
<summary><b>Configuration</b></summary>

File: `~/.sherlog/config.toml`. The `SHERLOG_API_KEY` environment variable overrides the file.

| Key | Default | Description |
|---|---|---|
| `provider` | `anthropic` | `anthropic` or `ollama` |
| `model` | `claude-sonnet-4-20250514` | Model to use |
| `api_key` | none | Anthropic key (not needed for Ollama) |
| `api_base_url` | `https://api.anthropic.com` | Endpoint or proxy |
| `max_log_lines` | `200` | Max lines sent for analysis |
| `theme` | `dark` | Terminal theme |
| `telemetry` | `false` | Opt-in anonymous usage stats |

</details>

<details>
<summary><b>Supported log formats</b></summary>

| Format | Detects |
|---|---|
| Stack traces | Python, Node.js, Rust, Java, Go |
| JSON logs | JSON-per-line with common field names |
| Syslog | Standard syslog lines |
| Kubernetes | `kubectl logs`, events, pod logs |
| Docker | `docker logs`, build output, compose |
| Terraform | Plan/apply errors with HCL references |
| GitHub Actions | `##[error]`, `::error::`, workflow logs |
| Custom | Anything, via WASM plugins |

</details>

<details>
<summary><b>Architecture</b></summary>

```
src/
├── cli.rs          Clap CLI definition
├── commands/       One module per command
├── ai/             Claude + Ollama clients (shared interface)
├── parsers/        Log parsers + plugin integration
├── plugins/        WASM plugin system (Extism)
├── context/        Project-type detection
├── config/         TOML config
├── storage/        SQLite history
├── telemetry/      Opt-in usage stats
└── output/         Terminal / JSON / CI formatters
```

Design notes: parsers implement a common `LogParser` trait (plugins are tried first, then built-ins); both AI backends share one interface; a single `OutputMode` enum drives terminal, JSON, and CI output.

</details>

## 🗺️ Roadmap

- [ ] Secret redaction before logs are sent to any AI backend
- [ ] Demo GIF and a benchmark of fix accuracy across models
- [ ] Prebuilt binaries and a Homebrew tap
- [ ] Publish to crates.io

Have an idea? [Open an issue](https://github.com/Prathmesh010190/sherlog/issues).

## 🤝 Contributing

PRs are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md).

## 📄 License

[MIT](LICENSE)