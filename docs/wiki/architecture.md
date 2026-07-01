# Architecture and Decisions (ADRs)

This document records the system architecture and Design Decisions (Architecture Decision Records).

## System Overview

**QuotaChecker-TUI** is a monitoring dashboard for AI agents. It operates by scanning local telemetry sources (SQLite databases, logs, and configuration files) to provide a real-time view of API quota usage across multiple assistants.

### Core Components

- **Scanner (`agent.rs`):** A background thread that periodically polls local databases (Codex, OpenCode, Zed), config/log files (Agy, Aider, Ollama, Continue, Cody, Supermaven), and telemetry logs to update the application state.
- **Config Manager (`config.rs`):** Handles persistence of user preferences and custom quota limits using JSON.
- **UI Engine (`ui.rs`):** A `ratatui`-based rendering engine that provides a multi-tab interactive interface.

## ADRs

### ADR 0001: Background Scanning Thread

**Status**: Accepted  
**Date**: 2026-06-09  

#### Context

Scanning multiple SQLite databases and large log files can be I/O intensive and may block the UI thread, leading to a sluggish experience.

#### Decision

Implement an asynchronous background thread that performs the scanning logic and communicates updates to the main thread via an MPSC channel.

#### Consequences

- **Positive**: UI remains responsive during heavy I/O operations.
- **Negative**: Increased complexity in state synchronization.

### ADR 0002: Manual JWT/Base64 Decoding

**Status**: Accepted (Pending Refactoring)
**Date**: 2026-06-09

#### Context

Dependencies should be kept minimal.

#### Decision

Implemented custom Base64 and JWT payload decoding to avoid external crate overhead for simple metadata extraction.

#### Consequences

- **Positive**: Zero extra dependencies for auth parsing.
- **Negative**: More code to maintain; potential for edge-case bugs.
