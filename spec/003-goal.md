# Goal

## North Star
Deliver a reliable, opinionated agent manager for developers who expect:
- A Rust-native backend that mirrors the possibilities of existing TypeScript/Node CLI tooling
- Seamless ACP integration with Gemini, Codex, and other CLIs (authentication + tool calls)
- Clear contributor documentation so new authors can add agents, hooks, and commands without guessing where the logic lives

## Key Results
1. **Core stability** – every agent/hook added to `crates/core/src/agents` or `hooks` compiles and passes `cargo check`. Background/task orchestration logic is self-contained in `features/background-agent` and can survive multiple agent executions.
2. **Documentation clarity** – `docs/*`, `spec/*`, and `skills/*` describe what the extension does, how skills map to code, and how high-level workflows (e.g., `/changelog`, `background-task`) operate.
3. **User workflows** – CLI commands documented in `docs/QUICKSTART.md` and `.gemini/commands/*` let users boot Gemini sessions, approve OAuth requests, and run slash commands (e.g., `/docs`, `/review`).

## Who benefits
- **Developers building agents** use `skills/` templates plus `AGENTS.md` summaries to understand naming conventions, triggers, and tool restrictions.
- **Core maintainers** rely on `crates/` to import auth flows, hook definitions, and MCP integrations while avoiding duplication between `adapters/acp` and `cli` binaries.
- **End users** can load the extension, pick an agent (Gemini/Codex/Qwen), and rely on ACP to handle streaming, tool calls, and notifications.

## Immediate actions for contributors
- Keep specs in sync with code: update `spec/001-concept.md`, `spec/002-roadmap.md`, and `spec/003-goal.md` whenever core modules move or extend.
- Use `cargo check -p bl1nk-core` as the gate before pushing changes, especially when touching `config`, `session`, or `features/background-agent`.
- Document APIs (e.g., `AGENTS.md`, `docs/AGENT_USAGE_GUIDE.md`) whenever a new hook/agent is added, and re-run `just run docs` if available.
