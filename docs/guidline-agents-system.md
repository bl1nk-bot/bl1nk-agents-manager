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
