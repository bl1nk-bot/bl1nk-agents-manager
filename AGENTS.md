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
