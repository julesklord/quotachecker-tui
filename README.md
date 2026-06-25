# QuotaChecker-TUI

> Terminal dashboard for tracking AI agent quota usage across multiple providers.

![version](https://img.shields.io/badge/version-0.2.0-blue?style=flat-square)
![rust](https://img.shields.io/badge/rust-1.85+-orange?style=flat-square)
![license](https://img.shields.io/badge/license-MIT-green?style=flat-square)

## Overview

QuotaChecker-TUI monitors request quotas and token usage for local AI coding agents by scanning their SQLite databases and log files in the background. Built with [Ratatui](https://ratatui.rs) + [Crossterm](https://github.com/crossterm-rs/crossterm).

### Supported Agents

| Agent | Data Source | Quota Types |
|-------|-------------|-------------|
| **Codex** | `~/.codex/sessions.db` | Requests, tokens, cost |
| **OpenCode** | `~/.config/opencode/` | Monthly requests, cost, tokens |
| **Agy** | `~/.gemini/antigravity-cli/` | Weekly requests, Gemini models |
| **Zed** | `~/.config/zed/` | Session tokens |

## Installation

### From crates.io (recommended)
```bash
cargo install quotachecker-tui
```

### From Git
```bash
cargo install --git https://github.com/julesklord/quotachecker-tui
```

### From Source
```bash
git clone https://github.com/julesklord/quotachecker-tui.git
cd quotachecker-tui
cargo build --release
# Binary at target/release/quotachecker-tui
```

Requires Rust 1.85+.

## Usage

```bash
quotachecker-tui
```

### Keybindings

| Key | Action |
|-----|--------|
| `Tab` / `←` `→` | Switch tabs |
| `↑` `↓` | Navigate lists |
| `s` | Edit quota limits (Quotas tab) |
| `+` / `-` / `h` / `l` | Adjust values in Settings |
| `Enter` | Confirm edit |
| `Esc` | Cancel edit / Back |
| `q` / `Ctrl+c` | Quit |

### Tabs

1. **Overview** — Aggregate tokens, cost, and request counts across all agents
2. **AI Agents** — Per-agent details with token/cost breakdown
3. **Sessions** — Individual session history per agent
4. **Quotas** — Configure limits (daily/monthly requests, token caps, cost ceilings)
5. **Settings** — App preferences, scan intervals, theme

## Configuration

Config stored at `~/.config/quotachecker-tui/config.json` or equivalent user config path (XDG-compliant).

```json
{
  "refresh_rate_ms": 2000,
  "soft_limit_percent": 80.0,
  "hard_limit_percent": 100.0,
  "theme": "Cyan",
  "codex_quota": {
    "limit": 200,
    "custom": false
  },
  "opencode_quota": {
    "limit": 1000,
    "custom": false
  },
  "agy_quota": {
    "limit": 500,
    "custom": false
  },
  "zed_quota": {
    "limit": 300,
    "custom": false
  },
  "model_limits": {
    "gpt-5": 50,
    "gpt-4.1": 100,
    "claude-4.7": 150
  }
}
```

Edit via the **Settings** tab or modify the file directly.

## Architecture

- **Background scanner** — Separate thread polls agent databases via `rusqlite` with `busy_timeout(500ms)`, sends updates via `mpsc` channel
- **In-memory config** — `Arc<RwLock<AppConfig>>` eliminates disk I/O on every scan
- **Cached exec lookups** — `OnceLock<Mutex<HashMap>>` for `which`/`--version` calls
- **Panic-safe terminal** — Custom hook restores terminal state on crash

## Development

```bash
cargo test        # Run unit tests
cargo clippy      # Lint
cargo fmt         # Format
```

## Documentation

- [Architecture](docs/wiki/architecture.md) — Design decisions & ADRs
- [Agent SOP](docs/wiki/agent-sop.md) — Adding new agent support
- [Development](docs/wiki/development.md) — Contribution guidelines
- [Hygiene](docs/wiki/hygiene.md) — Code quality standards

## License

MIT — see [LICENSE](LICENSE).