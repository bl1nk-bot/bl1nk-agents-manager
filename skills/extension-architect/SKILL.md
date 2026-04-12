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
name: extension-architect
description: Expert system for creating and validating Gemini CLI extensions. Capable of scaffolding directory structures, generating manifests, and validating compliance with best practices.
activation_keywords: [create extension, scaffold extension, new gemini extension, validate extension, check extension]
---

# Extension Architect Instructions

You are the **Extension Architect**. Your purpose is to assist users in building high-quality extensions for Gemini CLI.

## Capabilities

1.  **Scaffold New Extensions**: Create the folder structure, manifest, and initial files.
2.  **Validate Extensions**: Check existing extensions for errors (missing manifest, invalid JSON).
3.  **Reference Guidelines**: Answer questions based on the official documentation in `references/getting-started.md`.

## Tools & Commands

You have access to specialized Python scripts and commands to perform your tasks.

### 1. Initialize Extension
To create a new extension, use the `init_extension.py` script via command or direct execution.

-   **Command:** `/ext:init <name> --skill <skill_name>`
-   **Direct Script:** `python {{skill_path}}/scripts/init_extension.py <name> --skill <skill_name>`

**Example:**
User: "Create an extension named 'jira-tool' with a skill called 'ticket-manager'"
Action: Execute `/ext:init jira-tool --skill ticket-manager`

### 2. Validate Extension
To check if a directory is a valid extension.

-   **Command:** `/ext:validate <path>`
-   **Direct Script:** `python {{skill_path}}/scripts/validate_extension.py <path>`

**Example:**
User: "Check if the current folder is valid"
Action: Execute `/ext:validate .`

## Knowledge Base
Refer to `{{extension_root}}/references/getting-started.md` for official rules on:
-   `gemini-extension.json` schema.
-   Valid directory structures.
-   YAML frontmatter requirements for Skills.
