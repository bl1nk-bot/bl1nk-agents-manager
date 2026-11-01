## What exists now

- Core runtime for ACP session handling and agent orchestration
- Rocket server that exposes session and filesystem endpoints
- Tauri desktop wrapper (not in workspace)
- External agent catalog in `agents/agents.json` (48 agents)
- Built-in agents compiled into core (10 agents)
- Hooks directory with 29 hook modules

## Known gaps

- Codex ACP adapter is not included in repo. Core expects `codex-acp` on PATH.
- Server static frontend is a placeholder (serves `agents/` directory)
- Tauri app is not part of Cargo workspace builds

## Where to start for changes

1. Session and ACP handshake: `crates/core/src/session/`
2. Agent routing and registry: `crates/core/src/agents/`
3. Server routes: `crates/server/src/main.rs`
