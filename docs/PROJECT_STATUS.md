# Project status (February 7, 2026)

This document explains the current state of Bl1nk Agents Manager in plain
language. It is intended for readers who are not working on the codebase but
need to understand what is ready, what is in progress, and what is still
missing.

## What this project is
Bl1nk Agents Manager is a Gemini CLI extension and Rust backend that orchestrate
multiple AI agents and tools through MCP/ACP. The goal is to let different agent
CLIs (Gemini, Codex, Qwen, and others) work through one unified system so users
can switch agents without re‑configuring everything each time.

## What is working today
- **Extension shell exists**: `gemini-extension.json` is present and points to
  the Rust binary. Gemini CLI can discover the extension.
- **Core Rust modules exist**: `crates/core` contains modules for agents, hooks,
  MCP/ACP adapters, session handling, filesystem, search, and RPC.
- **Server crate exists**: `crates/server` provides the HTTP server shell for
  future remote scenarios.
- **Command and doc sets exist**: command definitions and documentation are
  present and being refreshed to match real runtime behavior.

## What is in progress
- **TypeScript → Rust parity**: large subsystems are still being ported from the
  original TypeScript logic into Rust.
- **Background agent system**: core types and task orchestration are being
  migrated, but end‑to‑end stability is not complete yet.
- **Config schema + loader alignment**: schema definitions are in place, but
  loader wiring and migration behavior still need validation against real usage.
- **ACP adapter normalization**: agent CLIs use ACP slightly differently. The
  unified adapter layer is being tightened to handle those differences.

## Known gaps and risk areas
- Some Rust modules compile but are not fully wired end‑to‑end.
- Authentication flows for some CLIs still require manual steps.
- The documentation set has partial drift from runtime behavior and is being
  updated to reflect the actual state.

## What non‑developers should expect right now
- You can explore the architecture and agent catalog.
- Many workflows are usable for experimentation but are not stable enough for
  production use.
- Some tasks require manual setup and troubleshooting.

## What this project is not yet
- A fully‑stable multi‑agent platform with complete automation for all CLIs.
- A fully‑documented, consumer‑ready product with guaranteed behavior across
  all providers.

## Near‑term milestones
1. Make `cargo check` and tests clean across the workspace.
2. Complete the remaining large Rust ports (background agents and config).
3. Validate ACP session lifecycle for Gemini/Codex/Qwen in the unified adapter.
4. Refresh documentation to match working behavior and verified flows.

## Where to look next
- `README.md` for the overview and features
- `docs/PROJECT_MAP.md` for the repo layout
- `docs/ARCHITECTURE.md` for system data flow
- `docs/QUICKSTART.md` for current setup steps
