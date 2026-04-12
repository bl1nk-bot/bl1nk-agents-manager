# Claude Command Best Practices
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

## 1. Naming Conventions
- **Verb-Noun**: `create-component`, `run-tests`.
- **Categories**: Use prefixes for grouping.
  - `gh:` for GitHub tasks.
  - `ci:` for continuous integration.
  - `sys:` for system maintenance.
  - `gen:` for code generation.

## 2. Tool Permissions (`allowed-tools`)
Restrict tool access to the minimum required for security:
- **Read-only**: `Read, LS`
- **File modification**: `Write`
- **Restricted Bash**: `Bash(git:*), Bash(npm:*)`
- **Full access** (use sparingly): `*`

## 3. Error Handling Workflow
1. **Pre-flight Checks**: Verify arguments and environment before acting.
2. **Graceful Fallback**: If a primary tool fails, suggest an alternative or explain why.
3. **Verification**: Always check if the action was successful (e.g., check if a file was created or a command returned exit code 0).

## 4. Documentation
Include examples in the command file so users know how to call it:
```markdown
## Examples:
- `/gen:component Header`
- `/gen:component Button --variant primary`
```
