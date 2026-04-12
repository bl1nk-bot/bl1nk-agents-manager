# Project summary
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
## What exists now

- Core runtime for ACP session handling and agent orchestration
- Rocket server that exposes session and filesystem endpoints
- Tauri desktop wrapper (not in workspace)
- External agent catalog in `agents/agents.json` (48 agents)
- Built-in agents compiled into core (10 agents)
- Hooks directory with 29 hook modules

## Known gaps

- Codex ACP adapter is not included in repo. Core expects `codex-acp` on PATH.
- Server static frontend is a placeholder (serves `agents/` directory)
- Tauri app is not part of Cargo workspace builds

## Where to start for changes

1. Session and ACP handshake: `crates/core/src/session/`
2. Agent routing and registry: `crates/core/src/agents/`
3. Server routes: `crates/server/src/main.rs`
