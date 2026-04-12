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

---
name: poe-bot-script-creator
description: "Expert system for designing and creating multi-bot management systems on Poe. Use for: architecting bot registries, creating bot factories, managing prompts/scripts, and implementing parallel/sequential bot execution."
---

# Poe Bot Script Creator

This skill provides a comprehensive framework for designing and implementing sophisticated bot management systems on the Poe platform. It follows a **Clean Architecture** approach to ensure scalability and maintainability when handling multiple bots.

## Core Architecture

The system is built around four primary pillars that work in harmony to manage the lifecycle and execution of Poe bots.

| Component | Description |
| :--- | :--- |
| **Bot Registry** | The central source of truth for all bot configurations, metadata, and active session states. |
| **Bot Creation Factory** | A specialized engine for instantiating bots from templates (Inline, Skill, Creative) with dynamic skill injection. |
| **Execution Engine** | The orchestrator responsible for parallel execution, sequential pipelines, and intelligent routing. |
| **Prompt & Script Manager** | A management layer for versioning prompts, validating parameters, and generating bot scripts. |

## Operational Workflows

### 1. Designing a Multi-Bot System
When starting a new project, always begin with the **Core** component. Define the `BotRegistry` schema first to ensure all subsequent components have a consistent data model to interact with.

### 2. Implementing Execution Strategies
Choose the appropriate execution mode based on the user's requirements:
- **Parallel**: Use when multiple perspectives or redundant checks are needed.
- **Sequential**: Use for complex tasks that require a multi-step pipeline (e.g., Research -> Analyze -> Report).
- **Router**: Use for general-purpose interfaces that need to delegate tasks to specialized sub-bots.

## Resources and References

To maintain a lean context, detailed specifications and templates are stored in the following reference files:

- **SDK Reference**: See `references/poe_sdk_reference.md` for API details on `poe.Message`, `poe.Attachment`, and `fastapi_poe`.
- **Architecture Spec**: See `references/architecture_spec.md` for deep dives into component responsibilities and data structures.
- **Bot Templates**: See `references/bot_templates.md` for pre-defined configurations for common bot types like Translators and Researchers.
- **Code Boilerplate**: Use `templates/poe_bot_boilerplate.py` as a starting point for new bot implementations.

## Best Practices

- **Clean Architecture**: Ensure strict separation between the execution logic and the Poe SDK integration layer.
- **Resource Management**: Implement load balancing and resource limits to prevent exhaustion when running multiple concurrent bots.
- **SSE & Protocols**: Prefer **Server-Sent Events (SSE)** and local execution protocols (`pmcp`, `rust acp`) over heavy containerization like Docker for a lighter footprint.

