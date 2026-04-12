# Python / SQLAlchemy Analysis
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

## 1. N+1 Query Detection

### Detection
- Look for access to relationships (e.g., `user.addresses`) inside a loop over results.
- Default loading in SQLAlchemy is usually lazy.

### Solutions
- **Eager Loading:** Use `joinedload` or `subqueryload` options in the query.
  ```python
  stmt = select(User).options(joinedload(User.addresses))
  ```
- **AsyncIO:** For `asyncpg`, ensure relationships are loaded via options or explicit `await` calls to avoid "Greenlet error" or implicit I/O blocking.

## 2. Session Management

- **Scope:** Ensure `Session` is scoped to the request (e.g., using `dependency_injector` or FastAPI dependency).
- **Commit/Rollback:** Verify explicit commit or context manager usage.
  ```python
  with Session(engine) as session:
      # ... operations
      session.commit()
  ```

## 3. Alembic Migrations

- **Autogenerate Review:** Always inspect `alembic revision --autogenerate` output. It may miss:
  - Sequence creation/renaming.
  - Triggers.
  - Complex constraint changes.
