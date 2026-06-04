# Changelog

All notable changes to this project will be documented in this file.
Format: [keepachangelog.com](https://keepachangelog.com) · Versioning: [semver.org](https://semver.org)

## [Unreleased]

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
