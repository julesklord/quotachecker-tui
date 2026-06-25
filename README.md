# QuotaChecker-TUI

[![Crates.io](https://img.shields.io/crates/v/quotachecker-tui.svg?style=flat-square)](https://crates.io/crates/quotachecker-tui)
[![Crates.io Downloads](https://img.shields.io/crates/d/quotachecker-tui.svg?style=flat-square)](https://crates.io/crates/quotachecker-tui)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg?style=flat-square)](LICENSE)

A terminal dashboard to track usage limits and token consumption of your local AI coding assistants.

![QuotaChecker-TUI Demo](docs/demo.gif)

## What it does

Tracks requests and token usage in the background by querying the local database and log files of your installed coding assistants.

### Supported Assistants

| Assistant | Data Source | Collected Metrics | Reset Freq |
| :--- | :--- | :--- | :--- |
| Codex | `~/.codex/state_5.sqlite` | Sessions, requests, tokens | Daily |
| OpenCode | `~/.local/share/opencode/opencode.db` | Sessions, requests, tokens, spent cost | Monthly |
| Agy | `~/.gemini/antigravity-cli/log/` | CLI prompts, command logs | Weekly |
| Zed | `~/.local/share/zed/threads/threads.db` | Active threads | Daily |

## Installation

### From crates.io (Recommended)
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
```
The compiled binary is located at `./target/release/quotachecker-tui`.

## Usage

Run the dashboard:
```bash
quotachecker-tui
```

### Keybindings

| Key | Action |
| :--- | :--- |
| `Tab` / `←` `→` | Switch tabs |
| `↑` `↓` | Navigate lists |
| `s` | Edit active assistant request limits |
| `+` / `-` | Modify values in Settings |
| `Enter` | Confirm and save inputs |
| `Esc` | Cancel modal |
| `r` | Force-trigger a background telemetry scan |
| `q` | Quit |

### Available Tabs

1. **Overview** — Aggregate costs, tokens, and requests across all assistants.
2. **AI Agents** — Versions, configurations, and quota breakdown for the selected assistant.
3. **Sessions** — Past sessions and telemetry logs.
4. **Quotas** — Usage gauges with warning thresholds.
5. **Settings** — Refresh intervals and visual themes.

## Configuration

The config file is located at `~/.config/quotachecker-tui/config.json`.

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

### Configuration Fields
- `refresh_rate_ms`: Delay in milliseconds between background scans.
- `soft_limit_percent`: Percentage where gauges turn yellow to warn the user.
- `hard_limit_percent`: Percentage where gauges turn red indicating limit is reached.
- `custom`: When set to `true`, the application respects your configured limit. When `false`, it defaults to the detected tier quota.

## Technical Details

### Telemetry & Tier Resolution
- **Codex**: Parses `~/.codex/auth.json` to extract the JWT `id_token`. Decodes the payload and reads `https://api.openai.com/auth` -> `chatgpt_plan_type`. Plans matching `free` resolve to `OAuthPersonal` (200 daily requests); others resolve to `OAuthEnterprise` (2000 daily requests).
- **OpenCode**: Searches for an active JSON key in `auth.json` across XDG config directories. Resolves the provider (`github-copilot`, `openai`, `anthropic`, `deepseek`, `google`). If Copilot or Anthropic is active, resolves to `Enterprise` (2000 monthly requests). Others resolve to `PersonalFree` (1000 monthly requests) or fallback to `Guest` (200 monthly requests).
- **Agy**: Checks the presence of the `agy` binary in `$PATH` and scans `.gemini/antigravity-cli/log/` logs. Resolves to `AdvancedCli` (500 weekly requests).
- **Zed**: Inspects `~/.local/share/zed/threads/threads.db`. Resolves to `OAuthPersonal` (300 daily requests).

### Proportional Model Quotas
Model quotas scale dynamically based on the resolved agent limit (`L`):
- **Codex** (Enterprise/Personal): `gpt-5` (0.25 × L), `gpt-4.1` (0.50 × L), `claude-4.7` (0.75 × L).
- **Codex** (LocalFree): `gpt-5` (0.20 × L), `gpt-4.1` (0.40 × L), `claude-4.7` (0.60 × L).
- **OpenCode** (Copilot Enterprise): `gpt-5` (0.25 × L), `gpt-4.1` (0.50 × L), `claude-4.7` (0.75 × L).
- **OpenCode** (Copilot PersonalFree/Guest): `gpt-5` (0.05 × L), `gpt-4.1` (0.10 × L), `claude-4.7` (0.15 × L).
- **Agy**: `Gemini 3.5 Flash` (3.00 × L), `Gemini 3.1 Pro` (0.10 × L).
- **Zed**: `claude-4.7` (0.50 × L).

## Architecture

- **Asynchronous Telemetry**: A background thread reads SQLite databases using a `500ms` busy timeout to avoid write locks on active AI tools.
- **In-Memory Cache**: Shared config uses `Arc<RwLock<AppConfig>>` to prevent constant disk I/O.
- **Panic Hook**: Restores terminal state if the application crashes unexpectedly.

## License

MIT License. See [LICENSE](LICENSE).