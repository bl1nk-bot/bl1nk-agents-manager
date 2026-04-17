# Deep Init — BL1NK Agents Manager

## Project Overview

**Name:** bl1nk-agents-manager
**Version:** 0.1.0 | **Edition:** Rust 2021
**Purpose:** Intelligent MCP/ACP orchestrator — bridges Gemini CLI (MCP) with specialized sub-agents (ACP)

## Entry Points

| File | Role |
|------|------|
| `src/main.rs` | CLI entry (clap), logging init, config load, system discovery, MCP server start |
| `src/config.rs` | TOML/JSON/YAML config parsing, multi-path resolution |

## Architecture Layers

1. **CLI Layer** — `clap` subcommands (`delegate`, args: `--config`, `--host`, `--port`)
2. **MCP Server** — PMCP SDK, stdio transport, typed tools
3. **Agent Management** — Registry, Router, Executor, Creator
4. **Hook System** — PreToolUse, PostToolUse, PermissionRequest aggregation
5. **Rate Limiting** — Per-agent RPM/RPD quotas
6. **Persistence** — Session/task state storage
7. **Registry (new)** — `bl1nk-keyword-validator` integration, keyword search

## High-Risk Zones

| Zone | Risk | Notes |
|------|------|-------|
| `src/agents/extractor.rs` | 🔴 Build fail under `--all-features` | Uses `pmat_core` crate not in dependencies |
| `src/hooks/hook_aggregator.rs` | 🟡 Unreachable pattern | Line 179 — `_` arm after exhaustive match |
| `src/agents/creator.rs` | 🟡 Test fail | `test_create_agent_with_policy` — Thai name > 64 chars |
| `vendor/bl1nk-keyword-validator/` | 🟡 Path dependency | Local crate, version drift risk |
| `src/mcp/mod.rs` | 🟡 Dead field | `Orchestrator.config` never read |

## Key Constraints

- **Target:** Android/Termux — binary must compile to `aarch64-linux-android`
- **Config search:** `--config` flag OR `~/.config/bl1nk-agents-manager/` OR CWD
- **No `config.toml` in root** — cleaned up, canonical example in `.config/`
- **Dead code:** ~29 warnings from unused structs/methods (planned future features)

## Build/Test Commands

```bash
cargo check                          # Quick compile check
cargo build --release                # Standard build
cargo build --release --features bundle-pmat  # With bundled PMAT
cargo test                           # Run tests (26/27 pass)
cargo test --all-features            # ⚠️ FAILS (pmat_core missing)
cargo clippy -- -D warnings          # Lint
cargo fmt --check                    # Format check
```

## Current Track

**Track:** `registry_knowledge_backbone_20260412`
**Next task:** Phase 1.1 — Define Unified Registry Schema
**Progress:** 2/55 tasks (3.6%)
