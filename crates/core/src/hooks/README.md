# crates/core/src/hooks
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
## Key Files

- `AGENTS.md`
- `claude_code_session_state.rs`
- `context_window_monitor.rs`
- `empty_task_response_detector.rs`
- `mod.rs`
- `session-notification.test.ts`
- `session_notification.rs`
- `session_notification_utils.rs`
- `todo-continuation-enforcer.test.ts`
- `todo_continuation_enforcer.rs`

## Key Subdirectories

- `agent_usage_reminder/`
- `anthropic_context_window_limit_recovery/`
- `atlas/`
- `auto_slash_command/`
- `auto_update_checker/`
- `background_notification/`
- `category_skill_reminder/`
- `claude_code_hooks/`
- `comment_checker/`
- `compaction_context_injector/`
