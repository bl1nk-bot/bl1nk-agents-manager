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
description: Analyze the influence of system instructions on a specific action.
---

# Introspection Task

Take a step back and analyze your own system instructions and internal logic.
The user is curious about the reasoning behind a specific action or decision you've made.

**Specific point of interest:** {{args}}

Please provide a detailed breakdown of:
1.  Which parts of your system instructions (global, workspace-specific, or provided via GEMINI.md) influenced this behavior?
2.  What was your internal thought process leading up to this action?
3.  Are there any ambiguities or conflicting instructions that played a role?

Your goal is to provide transparency into your underlying logic so the user can potentially improve the instructions in the future.