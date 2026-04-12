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

---
description: "Create plugin settings file with user preferences"
allowed-tools: ["Write", "AskUserQuestion"]
---

# Create Plugin Settings

This command helps users create a `.claude/my-plugin.local.md` settings file.

## Steps

### Step 1: Ask User for Preferences

Use AskUserQuestion to gather configuration:

```json
{
  "questions": [
    {
      "question": "Enable plugin for this project?",
      "header": "Enable Plugin",
      "multiSelect": false,
      "options": [
        {
          "label": "Yes",
          "description": "Plugin will be active"
        },
        {
          "label": "No",
          "description": "Plugin will be disabled"
        }
      ]
    },
    {
      "question": "Validation mode?",
      "header": "Mode",
      "multiSelect": false,
      "options": [
        {
          "label": "Strict",
          "description": "Maximum validation and security checks"
        },
        {
          "label": "Standard",
          "description": "Balanced validation (recommended)"
        },
        {
          "label": "Lenient",
          "description": "Minimal validation only"
        }
      ]
    }
  ]
}
```

### Step 2: Parse Answers

Extract answers from AskUserQuestion result:

- answers["0"]: enabled (Yes/No)
- answers["1"]: mode (Strict/Standard/Lenient)

### Step 3: Create Settings File

Use Write tool to create `.claude/my-plugin.local.md`:

```markdown
---
enabled: <true if Yes, false if No>
validation_mode: <strict, standard, or lenient>
max_file_size: 1000000
notify_on_errors: true
---

# Plugin Configuration

Your plugin is configured with <mode> validation mode.

To modify settings, edit this file and restart Claude Code.
```

### Step 4: Inform User

Tell the user:
- Settings file created at `.claude/my-plugin.local.md`
- Current configuration summary
- How to edit manually if needed
- Reminder: Restart Claude Code for changes to take effect
- Settings file is gitignored (won't be committed)

## Implementation Notes

Always validate user input before writing:
- Check mode is valid
- Validate numeric fields are numbers
- Ensure paths don't have traversal attempts
- Sanitize any free-text fields
