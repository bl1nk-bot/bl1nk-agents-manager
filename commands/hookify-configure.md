
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
# Configure Hookify Rules

**Load hookify:writing-rules skill first** to understand rule format.

Enable or disable existing hookify rules using an interactive interface.

## Steps

### 1. Find Existing Rules

Use Glob tool to find all hookify rule files:
```
pattern: ".claude/hookify.*.local.md"
```

If no rules found, inform user:
```
No hookify rules configured yet. Use `/hookify` to create your first rule.
```

### 2. Read Current State

For each rule file:
- Read the file
- Extract `name` and `enabled` fields from frontmatter
- Build list of rules with current state

### 3. Ask User Which Rules to Toggle

Use AskUserQuestion to let user select rules:

```json
{
  "questions": [
    {
      "question": "Which rules would you like to enable or disable?",
      "header": "Configure",
      "multiSelect": true,
      "options": [
        {
          "label": "warn-dangerous-rm (currently enabled)",
          "description": "Warns about rm -rf commands"
        },
        {
          "label": "warn-console-log (currently disabled)",
          "description": "Warns about console.log in code"
        },
        {
          "label": "require-tests (currently enabled)",
          "description": "Requires tests before stopping"
        }
      ]
    }
  ]
}
```

**Option format:**
- Label: `{rule-name} (currently {enabled|disabled})`
- Description: Brief description from rule's message or pattern

### 4. Parse User Selection

For each selected rule:
- Determine current state from label (enabled/disabled)
- Toggle state: enabled → disabled, disabled → enabled

### 5. Update Rule Files

For each rule to toggle:
- Use Read tool to read current content
- Use Edit tool to change `enabled: true` to `enabled: false` (or vice versa)
- Handle both with and without quotes

**Edit pattern for enabling:**
```
old_string: "enabled: false"
new_string: "enabled: true"
```

**Edit pattern for disabling:**
```
old_string: "enabled: true"
new_string: "enabled: false"
```

### 6. Confirm Changes

Show user what was changed:

```
## Hookify Rules Updated

**Enabled:**
- warn-console-log

**Disabled:**
- warn-dangerous-rm

**Unchanged:**
- require-tests

Changes apply immediately - no restart needed
```

## Important Notes

- Changes take effect immediately on next tool use
- You can also manually edit .claude/hookify.*.local.md files
- To permanently remove a rule, delete its .local.md file
- Use `/hookify:list` to see all configured rules

## Edge Cases

**No rules to configure:**
- Show message about using `/hookify` to create rules first

**User selects no rules:**
- Inform that no changes were made

**File read/write errors:**
- Inform user of specific error
- Suggest manual editing as fallback
