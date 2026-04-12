# Agent system guidelines
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
## Sources of truth

- Built-in agents: `crates/core/src/agents/`
- External agents: `agents/agents.json` + `agents/*.md`
- Skills: `skills/skills.json` + `skills/*/SKILL.md`
- Routing rules: `config/Config.toml`

## Routing flow

1. Task enters core
2. Router evaluates rules in `config/Config.toml`
3. Core selects built-in or external agent
4. ACP session runs the external CLI and streams output

## When to update docs

- Update agent docs when `agents/agents.json` changes
- Update API docs when `crates/server/src/main.rs` routes change
- Update quickstart when build or run commands change
