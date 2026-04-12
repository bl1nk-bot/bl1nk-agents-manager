# Real-World API Examples for GPT Actions
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

This document contains examples of OpenAPI specifications for real-world APIs.

## 1. Weather API (Simple GET)
```yaml
openapi: 3.1.0
info:
  title: Weather API
  version: 1.0.0
servers:
  - url: https://api.weather.com/v3
paths:
  /wx/conditions/current:
    get:
      operationId: getCurrentConditions
      parameters:
        - name: location
          in: query
          required: true
          schema:
            type: string
      responses:
        '200':
          description: Current weather conditions
```

## 2. Spotify API (OAuth 2.0)
```yaml
openapi: 3.1.0
info:
  title: Spotify Web API
  version: 1.0.0
servers:
  - url: https://api.spotify.com/v1
paths:
  /me/player/currently-playing:
    get:
      operationId: getCurrentlyPlaying
      responses:
        '200':
          description: Information about the currently playing track
components:
  securitySchemes:
    oauth2:
      type: oauth2
      flows:
        authorizationCode:
          authorizationUrl: https://accounts.spotify.com/authorize
          tokenUrl: https://accounts.spotify.com/api/token
          scopes:
            user-read-currently-playing: Read currently playing track
```
