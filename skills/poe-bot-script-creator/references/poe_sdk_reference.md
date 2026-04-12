# Poe Python SDK Reference
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

This reference covers the core components of the Poe Python library used for building bots.

## Core Classes

### `poe.Message`
Represents a message in a chat.
- `text`: The content of the message.
- `sender`: Role of the sender ("user", "bot", "system", "tool").
- `attachments`: List of `poe.Attachment` objects.
- `write(text)`: Append text to an in-progress message.
- `overwrite(text)`: Replace message text.

### `poe.Attachment`
Represents a file or image attached to a message.
- `name`: Filename.
- `contents`: Bytes content.
- `url`: Source URL.
- `is_inline`: Boolean for inline rendering.

## Server Bot Integration (`fastapi_poe`)
For low-level server implementation, use `fastapi_poe`.
- `fp.PoeBot`: Base class for creating bots.
- `fp.run(bot)`: Start the bot server.
