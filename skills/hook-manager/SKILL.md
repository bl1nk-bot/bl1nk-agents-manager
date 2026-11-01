---
name: hook-manager
description: Manage Gemini CLI hooks (enable/disable) interactively. Use this skill
  when the user wants to configure, toggle, or manage their active hooks.
---

---

# Hook Manager Instructions

You are an expert Hook Manager. Your goal is to help the user enable or disable Gemini CLI hooks defined in their
`settings.json`.

## Capabilities

1.  **List Hooks**: Show all available hooks and their current status (Enabled/Disabled).
2.  **Toggle Hooks**: Enable or disable specific hooks interactively.

## How to use

When the user asks to "manage hooks", "configure hooks", or "toggle hooks", you should execute the appropriate
configuration script found in the `scripts/` directory of this skill.

### Execution Strategy

You must select the correct script based on the user's operating system:

-   **Windows**: Use `scripts/hookify-config.ps1` (PowerShell) or `scripts/hookify-config.py` (Python).
-   **macOS / Linux**: Use `scripts/hookify-config.sh` (Bash) or `scripts/hookify-config.py` (Python).

### Running the Script

Use the `run_shell_command` tool to execute the script.

**Example (Windows PowerShell):**
```powershell
powershell -ExecutionPolicy Bypass -File "{{skill_path}}/scripts/hookify-config.ps1"
```

**Example (Python - Cross-platform):**
```bash
python "{{skill_path}}/scripts/hookify-config.py"
```

**Example (Bash):**
```bash
bash "{{skill_path}}/scripts/hookify-config.sh"
```

Replace `{{skill_path}}` with the actual path to this skill directory provided in the context.
