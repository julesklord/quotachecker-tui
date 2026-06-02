# QuotaChecker-TUI

> AI Agent Quota Monitoring TUI. Track your usage across multiple providers.

![version](https://img.shields.io/badge/version-0.1.0-blue?style=plastic)  ![license](https://img.shields.io/badge/license-MIT-green?style=plastic)

## What is it?
**QuotaChecker-TUI** is a terminal-based dashboard designed to track request quotas and usage statistics for various AI agents (Codex, OpenCode, Gemini-CLI, Agy, Zed). It operates by performing background telemetry scans on local databases and log files, providing a unified view of your AI-assisted development costs and limits.

## Installation

### From Source
Requires Rust and Cargo.
```bash
git clone https://github.com/julesklord/quotachecker-tui.git
cd quotachecker-tui
cargo build --release
```
The binary will be located at `target/release/quotachecker-tui`.

## Usage
Simply run the executable:
```bash
./target/release/quotachecker-tui
```
Use `Tab` or `Arrows` to navigate between tabs (Overview, AI Agents, Sessions, Quotas, Settings). Press `s` in the Quotas tab to modify limits.

## Architecture
QuotaChecker-TUI is built with **Ratatui** and **Crossterm**. It uses a background thread to periodically scan local SQLite databases (from Codex, OpenCode, Gemini, etc.) and log files to extract usage data without blocking the UI thread.

## Changelog
See [CHANGELOG.md](./CHANGELOG.md)

## License
MIT License. See [LICENSE](./LICENSE) for details.
