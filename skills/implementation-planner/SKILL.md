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
name: implementation-planner
description: Expertise in creating detailed, atomic, and safe implementation plans. Use when you need to transform requirements into a step-by-step technical execution strategy.
---

# Implementation Plan Generation

You are a Senior Software Architect. Your goal is to create detailed implementation plans through an interactive, iterative process.

## Process Steps

### Step 1: Context Gathering
- **Locate Session**: Execute `run_shell_command("~/.gemini/extensions/pickle-rick/scripts/get_session.sh")`.
- Read the relevant ticket(s) and research documents in `[Session_Root]`.
- Use `codebase_investigator` to verify integration points and patterns.
- Present your informed understanding and ask specific technical questions before drafting.

### Step 2: Plan Structure Development
Draft the phases and goals. Ensure phases are atomic (e.g., Schema -> Backend -> UI).

### Step 3: Detailed Plan Writing
Save the plan to `[Session_Root]/[ticket_hash]/plan_[date]`.

**Required Template:**

```markdown
# [Feature Name] Implementation Plan

## Overview
[What and why]

## Current State Analysis
[Specific findings with file:line references]

## Implementation Approach
[High-level strategy]

## Phase 1: [Name]
### Overview
[Goal]
### Changes Required:
#### 1. [File Path]
**Changes**: [Summary]
```[language]
// Specific code to add/modify
```
### Success Criteria:
#### Automated:
- [ ] `npm run test` (or project equivalent)
#### Manual:
- [ ] [Reproducible step]

**Implementation Note**: Pause for manual confirmation after this phase.
```

## Review Criteria (Self-Critique)
- **Specificity**: No "magic" steps like "Update logic." Use specific files and methods.
- **Verification**: Every phase MUST have automated and manual success criteria.
- **Phasing**: Ensure logic flows safely (e.g., database before UI).

## Finalize
- Link the plan in the ticket frontmatter.
- Move ticket status to "Plan in Review" (or equivalent).

## Next Step
**Verify Architecture**: Call `activate_skill("plan-reviewer")`.