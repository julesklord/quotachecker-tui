# Changelog

All notable changes to this project will be documented in this file.
Format: [keepachangelog.com](https://keepachangelog.com) · Versioning: [semver.org](https://semver.org)

## [Unreleased]

## [0.2.0] - 2026-06-25

### Added
- New unit test module (`src/tests.rs`) covering Base64 decoding, JWT parsing, and configuration defaults.
- Type alias `ScanResult` for better code readability in background scanning threads.
- Proportional dynamic model limits mapping for Codex, OpenCode, Agy, and Zed based on user tiers and configuration.
- Support for `custom` quota flag in configuration to track manually overridden limits from the TUI.

### Changed
- Refactored `agent.rs` to use idiomatic Rust patterns (`saturating_sub`, `is_multiple_of`, `vec!`).
- Updated architecture documentation (`docs/wiki/architecture.md`) with current design decisions and ADRs.
- Improved development and hygiene guides in the wiki to reflect strict quality mandates.
- Removed Gemini-CLI support completely from the application (removed logic, configuration parameters, and UI tables).

### Fixed
- Resolved 23 code quality issues identified by Clippy (linting).
- Fixed potential logic errors in Base64 padding and saturating arithmetic.
- Cleaned up syntax garbage and formatting issues in `ui.rs` and `main.rs`.

## [0.1.0] - 2026-06-04

### Added

- Updated TUI themes to use `Color::Reset` for backgrounds, preserving the terminal's native background colors and transparency.
- Added support for `QuotaType::Monthly` to track billing and token quotas on a monthly frequency.
- Implemented dynamic monthly reset calculation targeting the 1st of the next calendar month.
- Integrated live token and cost telemetry inside Codex and OpenCode by querying their respective SQLite databases.
- Made OpenCode use Monthly quota tracking globally, dynamically querying the last active provider (e.g., OpenAI vs GitHub Copilot) from its SQLite database to apply cost and subscription rules.
- Updated Overview tab stats boxes to display Total Tokens Used and Cumulative spend across all active assistants.
- Enhanced AI Agents details panel to dynamically present token consumption and cost details.
- Standardized default Pro model limits for Gemini to 50 requests/day matching actual Google AI Studio free tier limits.
- Applied FMG Repository Development Standard.
- Initial project structure with TUI for monitoring AI agent quotas.
