# Bot Management System Architecture Specification
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

This document defines the clean architecture for the Multi-Bot Management System on Poe.

## 1. Core Components

| Component | Responsibility | Key Features |
| :--- | :--- | :--- |
| **Bot Registry** | Centralized storage for bot metadata and state. | CRUD operations, session tracking, skill inventory. |
| **Bot Creation Factory** | Logic for instantiating different bot types. | Template-based creation, dynamic skill injection. |
| **Execution Engine** | Orchestrates bot runs and resource allocation. | Parallel/Sequential execution, Load balancing. |
| **Prompt & Script Manager** | Manages prompt templates and script generation. | Real-time editing, versioning, parameter validation. |

## 2. Bot Types

- **Inline Bots**: Single-purpose, quick execution (e.g., Translator, Formatter).
- **Skill Bots**: Multi-capability bots composed of multiple skills.
- **Creative Bots**: Specialized for content generation and ideation.

## 3. Execution Strategies

- **Parallel**: Run multiple bots simultaneously and merge results.
- **Sequential (Pipeline)**: Chain bots where output of one is input to next.
- **Router**: Intelligent intent detection to select the best bot.
