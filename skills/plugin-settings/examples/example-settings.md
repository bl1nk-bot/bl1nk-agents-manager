# Example Plugin Settings File
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

## Template: Basic Configuration

**.claude/my-plugin.local.md:**

```markdown
---
enabled: true
mode: standard
---

# My Plugin Configuration

Plugin is active in standard mode.
```

## Template: Advanced Configuration

**.claude/my-plugin.local.md:**

```markdown
---
enabled: true
strict_mode: false
max_file_size: 1000000
allowed_extensions: [".js", ".ts", ".tsx"]
enable_logging: true
notification_level: info
retry_attempts: 3
timeout_seconds: 60
custom_path: "/path/to/data"
---

# My Plugin Advanced Configuration

This project uses custom plugin configuration with:
- Standard validation mode
- 1MB file size limit
- JavaScript/TypeScript files allowed
- Info-level logging
- 3 retry attempts

## Additional Notes

Contact @team-lead with questions about this configuration.
```

## Template: Agent State File

**.claude/multi-agent-swarm.local.md:**

```markdown
---
agent_name: database-implementation
task_number: 4.2
pr_number: 5678
coordinator_session: team-leader
enabled: true
dependencies: ["Task 3.5", "Task 4.1"]
additional_instructions: "Use PostgreSQL, not MySQL"
---

# Task Assignment: Database Schema Implementation

Implement the database schema for the new features module.

## Requirements

- Create migration files
- Add indexes for performance
- Write tests for constraints
- Document schema in README

## Success Criteria

- Migrations run successfully
- All tests pass
- PR created with CI green
- Schema documented

## Coordination

Depends on:
- Task 3.5: API endpoint definitions
- Task 4.1: Data model design

Report status to coordinator session 'team-leader'.
```

## Template: Feature Flag Pattern

**.claude/experimental-features.local.md:**

```markdown
---
enabled: true
features:
  - ai_suggestions
  - auto_formatting
  - advanced_refactoring
experimental_mode: false
---

# Experimental Features Configuration

Current enabled features:
- AI-powered code suggestions
- Automatic code formatting
- Advanced refactoring tools

Experimental mode is OFF (stable features only).
```

## Usage in Hooks

These templates can be read by hooks:

```bash
# Check if plugin is configured
if [[ ! -f ".claude/my-plugin.local.md" ]]; then
  exit 0  # Not configured, skip hook
fi

# Read settings
FRONTMATTER=$(sed -n '/^---$/,/^---$/{ /^---$/d; p; }' ".claude/my-plugin.local.md")
ENABLED=$(echo "$FRONTMATTER" | grep '^enabled:' | sed 's/enabled: *//')

# Apply settings
if [[ "$ENABLED" == "true" ]]; then
  # Hook is active
  # ...
fi
```

## Gitignore

Always add to project `.gitignore`:

```gitignore
# Plugin settings (user-local, not committed)
.claude/*.local.md
.claude/*.local.json
```

## Editing Settings

Users can edit settings files manually:

```bash
# Edit settings
vim .claude/my-plugin.local.md

# Changes take effect after restart
exit  # Exit Claude Code
claude  # Restart
```

Changes require Claude Code restart - hooks can't be hot-swapped.
