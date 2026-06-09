# Development Guide

## Prerequisites

- **Rust:** Latest stable (v1.75+ recommended for `is_multiple_of` and other recent features).
- **Cargo:** Dependency manager and build tool.
- **SQLite:** Required for scanning agent databases (bundled via `rusqlite`).

## Local Setup

1. Clone the repository: `git clone <repo-url>`
2. Install dependencies and build: `cargo build`
3. Running the dashboard: `cargo run`

## Quality Control & Verification

We maintain high code quality standards. Every contribution must pass:

- **Linting:** `cargo clippy -- -D warnings` (Zero warnings allowed).
- **Formatting:** `cargo fmt --all -- --check` (Adhere to standard Rust style).
- **Testing:** `cargo test` (Validate logic and regressions).

## Project Structure

- `src/main.rs`: Application entry point and event loop.
- `src/agent.rs`: Scanners and telemetry logic.
- `src/ui.rs`: UI rendering and component definitions.
- `src/config.rs`: Configuration management.
- `src/tests.rs`: Unit testing module.
