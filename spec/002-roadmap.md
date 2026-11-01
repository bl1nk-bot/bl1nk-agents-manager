# Roadmap

## Themes
1. **Stabilize the Rust core** (now) – finish rewriting TypeScript orchestrator, hook, and session logic
   inside `crates/core`. Focused folders: `agents/`, `features/background-agent`, `config/`, `mcp/`, and `session/`.
2. **Unify ACP adapters** (next sprint) – ensure `adapters/acp` + `cli` entrypoints (e.g., `run_gemini`) can
   correctly translate between ACP payloads and the Rust runtime, including tool calls, prompts, and
   authentication flows.
3. **Build reliable workflows** – script tests for agent onboarding via `skills/`, provide CLI cheatsheets, and
   document `docs/PROJECT_STATUS.md` plus `spec/` so contributors understand where logic lives.

## Milestones (Feb 2026)
| Milestone | Status | Notes |
| --- | --- | --- |
| Background agent parity | In review | `features/background-agent` now compiles and handles queueing/polling; remaining tasks: notification hooks and MCP calls validation.
| Config/loader + migration | In progress | `config/loader.rs` handles JSONC detection; next step is aligning with `spec/configs` layout and hooking into nightly migration scripts.
| Docs refresh | Ongoing | `docs/AGENT_*`, `docs/ARCHITECTURE.md`, and `spec/` files are being rewritten to explain the new directory layout.

## Upcoming work (ordered)
1. Finish ACP handshake (OAuth flow + `GeminiBackend` event emission). Check points: `session/mod.rs`, `adapters/acp/`, `cli/` commands.
2. Expand hook coverage (e.g., `background_notification`, `task_resume_info`) and document in `crates/core/src/hooks/*/AGENTS.md`.
3. Map command/skill ownership so Prism tasks (e.g., `docs-writer`, `code-reviewer`) can run without manual prompts.
4. Harden tests: add unit/behavioral tests for `features/background-agent` and `session` flows before merging into `main`.

## Success checkpoints
- `cargo check -p bl1nk-core` plus `cargo fmt` run successfully without warnings from the highlighted modules.
- Documentation (spec, docs/AGENT*) matches the code location of agents/hooks being changed.
- The extension shell (`skills/`, `.gemini/commands/`) smoothly loads skills such as `docs-writer` and `pr-creator`.
