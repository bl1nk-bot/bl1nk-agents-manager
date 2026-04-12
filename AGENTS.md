# Repository Guidelines
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
## Project Structure & Module Organization
- `crates/core/`: Core runtime (agents, hooks, session, MCP, filesystem, routing).
- `crates/server/`: HTTP/WebSocket server exposing core capabilities.
- `agents/`: Agent definitions and `agents.json` registry.
- `skills/`: Skill directories plus `skills.json` registry.
- `commands/`: CLI command definitions.
- `config/Config.toml`: Default routing and agent configuration.
- `docs/`: Architecture and agent standards.
- `docs/PROJECT_MAP.md`: Auto-generated project map (run the update script to refresh).

## Build, Test, and Development Commands
- `just build`: Release build.
- `just run`: Run the server binary.
- `just dev`: Hot-reload development mode.
- `just test`: Run all tests.
- `just check`: Quick compile check.
- `just fmt`: Format Rust code.
- `just clippy`: Lint with strict warnings.
- `just validate-agents`: Validate agent metadata.
- `just verify`: Full workflow (fmt, clippy, check, validate-agents, audit skills).
- `python3 scripts/update_project_map.py`: Regenerate `docs/PROJECT_MAP.md`.

## Coding Style & Naming Conventions
- Rust uses `cargo fmt` and `cargo clippy --all-features`.
- Agent IDs and filenames are kebab-case (e.g., `code-reviewer`).
- Agent metadata must follow `docs/AGENT_SPECIFICATION.md`.
- Skills use `SKILL.md` and should be kebab-case folders.

## Testing Guidelines
- Primary tests are Rust tests via `cargo test`/`just test`.
- Hook tests: `cargo test --package bl1nk-core hooks`.
- Validate agents with `just validate-agents` before changes.

## Commit & Pull Request Guidelines
- No explicit commit style found in repo; use clear, imperative messages.
- PRs should describe changes, affected modules, and validation commands run.

## Architecture Notes
- MCP tools live in `crates/core/src/mcp/`.
- ACP protocol types live in `crates/core/src/adapters/acp/`.
- Session lifecycle is in `crates/core/src/session/`.
