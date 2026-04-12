# Quickstart
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
## Prerequisites

- Rust toolchain (stable)
- External CLI(s) for ACP (Gemini, Qwen, LLxprt)
- Codex ACP adapter if you plan to run Codex

## Build and run server

```bash
cargo build -p bl1nk-server
cargo run -p bl1nk-server
```

## Run core tests

```bash
cargo test -p bl1nk-core -- --test-threads=1
```

## Start a local session (dev)

```bash
cargo run -p bl1nk-core --bin run_gemini
cargo run -p bl1nk-core --bin run_codex
```
