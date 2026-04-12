# Interactive Bash Session Agent
## 📌 Project Status (Feb 7, 2026)

Bl1nk Agents Manager is in active development and is not feature‑complete yet.
This repo contains a working extension shell and a Rust core that is being
brought to feature parity with existing TypeScript logic.

**What works now**
- Extension manifest and Gemini CLI scaffolding are present.
- Core Rust modules exist for agents, hooks, MCP/ACP, sessions, and RPC.
- Command and documentation sets are present (currently being refreshed).

**In progress**
- TypeScript → Rust parity for large subsystems (background agents, config,
  ACP normalization).
- End‑to‑end session flows for Gemini/Codex/Qwen within a unified adapter.
- Validation of hook behavior and task orchestration across agents.

**Known gaps**
- Some Rust modules compile but are not fully wired end‑to‑end.
- Configuration loading/migration is still being aligned to actual runtime.
- Authentication flows for some CLIs still require manual steps.

**What to expect right now**
- You can explore the architecture, commands, and agent catalogs.
- Some workflows will still require manual setup or troubleshooting.

For a complete non‑developer overview, see `docs/PROJECT_STATUS.md`.
## Purpose

The Interactive Bash Session agent handles interactive bash sessions, including:
- TMUX session management
- Interactive command execution
- Session state tracking
- Terminal I/O handling

## Components

- `constants.rs`: Constant definitions for bash session operations
- `index.rs`: Main entry point and hook registration
- `storage.rs`: Session state storage and retrieval
- `types.rs`: Type definitions and interfaces

## Usage

This agent is automatically invoked when interactive bash operations are performed. It manages TMUX sessions and ensures proper state tracking for interactive terminal operations.

## Configuration

The agent uses default session management settings and does not require specific configuration.
