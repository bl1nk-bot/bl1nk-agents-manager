## High-level flow

1. Client calls server API or Tauri command
2. Server uses `GeminiBackend` (core) to start a session
3. Core spawns an external CLI and performs ACP handshake
4. Session streams updates through internal events
5. Server forwards events over WebSocket

## Core modules

- `session/` spawn CLIs, ACP handshake, IO loop
- `agents/` built-in agents, registry, router
- `adapters/acp/` ACP types and normalized updates
- `events/` event payloads and emitters
- `filesystem/`, `projects/`, `search/` local services

## External CLIs

- Gemini/Qwen/LLxprt run with ACP enabled
- Codex requires an ACP adapter binary named `codex-acp`
