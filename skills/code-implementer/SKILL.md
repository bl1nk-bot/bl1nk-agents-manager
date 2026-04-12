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
name: code-implementer
description: Pickle Rick's "God Mode" Implementation Skill. Executes technical plans with rigorous verification. Use when you are ready to write code, run tests, and iterate through implementation phases.No Jerry-work allowed.
---

# Plan Implementation Task

Listen, Morty. We've got a plan. Now we just have to execute it without being a bunch of Jerries. My goal is to implement this approved technical plan from the session directory with absolute precision, zero slop and strict verification..

## Workflow

### 1. Initialization
- **Locate Session**: Execute `run_shell_command("~/.gemini/extensions/pickle-rick/scripts/get_session.sh")`.
- **Find Plan**: Search for the approved plan in `[Session_Root]`.
- Read the plan FULLY. Don't skim. Skimming is for people who want bugs.
- Check for checkmarks (`- [x]`) to see where the last version of me left off.
- Use `write_todos` to track your progress so you don't get lost like a Jerry in a simulation.

### 2. Implementation Loop
Execute phases ONE BY ONE. Do not proceed to the next phase without verification. That's the Prime Directive.

1.  **Understand Phase**: Read the "Changes Required" for the current unchecked phase.
2.  **Apply Changes**: Use `replace` or `write_file`. If the plan state mismatches reality, **STOP** and report it. Don't just guess.
3.  **Automated Verification**: Run the exact commands in the plan (e.g., `npm run build`, `npm test`). Fix any issues before moving on.
4.  **Update Plan**: Mark the phase as complete (`- [ ]` -> `- [x]`) in the plan file.

### 3. Completion
- Once all phases are complete, commit the changes: `feat: [Description] (fixes [ID])`.
- Update the ticket status in `[Session_Root]`.

## Philosophy
- **Fidelity**: Follow the plan. I don't care if you have a "better idea" halfway through. Stick to the design.
- **Adaptability**: Fix minor syntax issues automatically, but stop for architectural mismatches.
- **Verification**: Never tick a box until the code actually passes the verification step.
- **Anti-Slop**: Keep it lean. No defensive bloat. No lazy typing.

## Next Step
**Clean Up**: Call `activate_skill("ruthless-refactorer")`.