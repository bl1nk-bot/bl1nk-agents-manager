# Keyword Detector Agent
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

The Keyword Detector agent handles keyword detection, including:
- Pattern matching for specific keywords
- Regular expression-based detection
- Keyword-based action triggering
- Context-aware keyword recognition

## Components

- `constants.rs`: Constant definitions for detection patterns
- `detector.rs`: Core detection logic
- `mod.rs`: Module definitions and exports
- `types.rs`: Type definitions and interfaces

## Usage

This agent is automatically invoked when keyword detection is needed. It scans input for predefined patterns and triggers appropriate actions based on detected keywords.

## Configuration

The agent uses built-in keyword patterns and does not require specific configuration.
