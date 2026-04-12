# Anthropic Context Window Limit Recovery Agent
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

The Anthropic Context Window Limit Recovery agent handles situations where Anthropic model requests exceed context window limits, providing:
- Automatic token limit error detection
- Intelligent content truncation strategies
- Recovery mechanisms to continue operations
- Preservation of important context information

## Components

- `executor.rs`: Core execution logic for recovery operations
- `index.rs`: Main entry point and hook registration
- `parser.rs`: Token limit error parsing and analysis
- `pruning_deduplication.rs`: Content deduplication and optimization
- `pruning_types.rs`: Type definitions for pruning operations
- `storage.rs`: Storage and retrieval of truncated content
- `types.rs`: Core type definitions and interfaces

## Usage

This agent is automatically invoked when Anthropic model requests encounter context window limit errors. It attempts to recover by intelligently truncating less important content while preserving critical context.

## Configuration

The agent uses built-in recovery strategies and does not require specific configuration. It operates based on detected token limit errors.
