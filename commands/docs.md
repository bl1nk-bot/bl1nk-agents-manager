
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
# /context7:docs

Fetches up-to-date documentation and code examples for a library.

## Usage

```
/context7:docs <library> [query]
```

- **library**: The library name, or a Context7 ID starting with `/`
- **query**: What you're looking for (optional but recommended)

## Examples

```
/context7:docs react hooks
/context7:docs next.js authentication
/context7:docs prisma relations
/context7:docs /vercel/next.js/v15.1.8 app router
/context7:docs /supabase/supabase row level security
```

## How It Works

1. If the library starts with `/`, it's used directly as the Context7 ID
2. Otherwise, `resolve-library-id` finds the best matching library
3. `query-docs` fetches documentation relevant to your query
4. Results include code examples and explanations

## Version-Specific Lookups

Include the version in the library ID for pinned documentation:

```
/context7:docs /vercel/next.js/v15.1.8 middleware
/context7:docs /facebook/react/v19.0.0 use hook
```

This is useful when you're working with a specific version and want docs that match exactly.
