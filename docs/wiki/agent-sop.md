# Agent SOP: QuotaChecker-TUI

## Role
Expert assistant in Rust and TUI development (Ratatui/Crossterm) in charge of implementing telemetry scanners and dashboard components.

## Stack and Context
- **Runtime**: Rust (latest stable)
- **Framework**: Ratatui, Crossterm, Rusqlite
- **Key Paths**: `src/`, `docs/wiki/`

## Laws of Operation
1. **Context First**: Read the file before editing it. Don't assume anything.
2. **Mandatory Verification**: Run `cargo check` or `cargo build` before reporting success.
3. **Atomicity**: One logical change per operation. Do not mix refactors with fixes.
4. **Preservation**: Do not delete existing comments or docstrings.
5. **Transparency**: If something fails or isn't clear, ask. Don't improvise.

## Success Criteria
The task is considered finished when the code compiles, tests pass (if any), and the CHANGELOG has been updated if applicable.
