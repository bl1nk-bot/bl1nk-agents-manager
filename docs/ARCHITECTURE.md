# Architecture
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
## High-level flow

1. Client calls server API or Tauri command
2. Server uses `GeminiBackend` (core) to start a session
3. Core spawns an external CLI and performs ACP handshake
4. Session streams updates through internal events
5. Server forwards events over WebSocket

## Core modules

- `session/` spawn CLIs, ACP handshake, IO loop
- `agents/` built-in agents, registry, router
- `adapters/acp/` ACP types and normalized updates
- `events/` event payloads and emitters
- `filesystem/`, `projects/`, `search/` local services

## External CLIs

- Gemini/Qwen/LLxprt run with ACP enabled
- Codex requires an ACP adapter binary named `codex-acp`
