# Bl1nk Agents Manager Workflows
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

## Quick Paths

- Agents: `agents/`, registry `agents/agents.json`
- Commands: `commands/`, `commands/agent/`
- Hooks: `crates/core/src/hooks/`
- Config and routing: `config/Config.toml`
- Skills: `skills/`, registry `skills/skills.json`
- Docs: `docs/AGENT_GUIDE.md`, `docs/AGENT_SPECIFICATION.md`, `docs/AGENT_MAINTENANCE.md`, `docs/ARCHITECTURE.md`, `docs/AGENT_USAGE_GUIDE.md`

## Agent Checklist

1. Create or update `agents/<id>.md` with YAML frontmatter and a clear system prompt.
2. Update `agents/agents.json` to match `id`, `name`, `file`, `category`, `description`, and any optional fields like `use_cases` or `tags`.
3. If adding a new agent, update `docs/AGENT_CATALOG.md` and `docs/AGENT_USAGE_GUIDE.md` where appropriate.
4. Run `just validate-agents` or `python3 scripts/validate_agents.py`.
5. If the validator fails, fix with `python3 scripts/fix_agents.py` and re-run validation.

## Command Checklist

1. Add or update a TOML definition under `commands/` or `commands/agent/`.
2. Use existing command files as patterns for `description`, `tags`, `complexity`, `category`, `outputs`, and `prompt`.
3. Keep prompts explicit about which tool to use and how to output results.

## Hook Checklist

1. Create a new module under `crates/core/src/hooks/`.
2. Register the module and re-export it in `crates/core/src/hooks/mod.rs`.
3. Keep hook logic single-purpose and non-blocking.
4. Test with `cargo test --package bl1nk-core hooks` or `just test`.

## Routing And Config Checklist

1. Edit `config/Config.toml` routing rules or agent entries as needed.
2. If routing behavior changes, confirm priority ordering and keyword coverage.
3. Verify any Rust structs or defaults that mirror config updates.

## Skill Checklist

1. Add a new skill folder under `skills/` with `SKILL.md`.
2. If the skill registry is used, update `skills/skills.json`.
3. Keep skill metadata aligned with the tag schema in `skills/skills.json`.
