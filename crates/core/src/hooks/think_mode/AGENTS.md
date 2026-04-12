# Think Mode Agent
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

The Think Mode agent handles thinking block operations, including:
- Thinking block detection and validation
- Cognitive processing mode management
- Thinking block injection and extraction
- Mode-specific processing rules

## Components

- `constants.rs`: Constant definitions for think mode operations
- `detector.rs`: Thinking block detection logic
- `index.rs`: Main entry point and hook registration
- `switcher.rs`: Mode switching logic
- `types.rs`: Type definitions and interfaces

## Usage

This agent is automatically invoked when thinking blocks are detected in input or when cognitive processing modes need to be managed. It validates thinking block structure and manages mode transitions.

## Configuration

The agent uses built-in thinking block patterns and does not require specific configuration.
