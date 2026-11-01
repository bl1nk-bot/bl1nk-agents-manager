# Concept

## Purpose
Bl1nk Agents Manager stitches together a Rust-based backend with ACP-compatible external agents
and the existing VS Code/extension shell so users can run Gemini, Codex, and Qwen workflows from a
single experience.

## Current Architecture (Feb 7, 2026)
- **Rust core (`crates/core/src/`)**: Implements agents, hooks, session management, MCP/ACP adapters,
  and filesystem helpers while exposing a thin CLI/adapter layer in `crates/core/src/adapters`.
- **Features**: The new `features/background-agent` module replicates the TypeScript background-task
  orchestration (concurrency manager, polling, notifications) in Rust so core logic can run server-side
  without depending on the frontend runtime.
- **Agents**: Agent definitions live under `crates/core/src/agents` (e.g., `atlas`, `liaison`, planners).
  Hooks (e.g., `task_resume_info`, `background_notification`) live under `crates/core/src/hooks` and are
  invoked through centralized registries and `AGENTS.md` guidance.
- **Extension shell (`skills/`, `.gemini/`, `commands/`)**: Provides commands, skills, and references that the
  CLI or community contributes to on top of the core runtime.
- **Documentation + assets (`docs/`, `spec/`, `README.md`)**: Capture the agent catalog, workflows, and
  architecture perspectives that external contributors need to follow.

## Tech Stack & Integrations
- **Rust** for the core agent runtime, background manager, session lifecycle, and configuration/migration logic.
- **ACP / MCP** adapters for Gemini/Codex/Qwen living under `crates/core/src/adapters`. These expose
  JSON-RPC helpers to talk to `gemini`, `codex`, and other CLI binaries.
- **Skill system** built on `.gemini/skills` + `skills/` directory so that Codex or Gemini can load instructions
  (e.g., docs-writer, code-reviewer, pr-creator) without code changes.
- **Hook + CLI scaffolding**: Scripts in `scripts/` and `commands/` define reusable operations while `justfile`
  and `package.json` support running lint/test/build steps across the workspace.

## What you can do today
1. Explore `docs/` to understand existing agents, workflows, and CLI cheatsheets.
2. Run `cargo check -p bl1nk-core` to validate the Rust backend and review `crates/core/src/features` for
   ongoing TypeScript → Rust rewrites such as `background-agent`.
3. Read `skills/` and `.gemini/skills` to see how prompts, hooks, and commands are packaged for hybrid clients.

## Gaps & Next Priorities
- Session-level flows still need polishing for authentication (OAuth approval handing) and tool chaining.
- Configuration migration around `config/loader.rs` is being lined up with `spec/` docs so new agent hooks work
  without manual edits.
- Documentation across `docs/` and `spec/` must stay accurate as Rust modules replace TypeScript logic.
