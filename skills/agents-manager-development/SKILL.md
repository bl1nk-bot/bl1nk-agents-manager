---
name: agents-manager-development
description: Comprehensive workflow for developing and maintaining the Bl1nk Agents
  Manager extension. Use when the user asks to add or update system agents, commands,
  hooks, routing rules, configuration, skills, or build/test tasks in this repository.
---

---

# Bl1nk Agents Manager Development

## Overview

Provide a focused workflow for contributing to the Bl1nk Agents Manager extension. Cover agent creation and maintenance,
command definitions, hook development, routing/config updates, and build/test procedures specific to this repo.

## Scope And Triage

1. Identify the change type.
2. Follow the matching workflow and update all required registries and docs.
3. Run the smallest validation or test command that proves correctness.

Change types:
- Agents and registry metadata
- Commands and command prompts
- Hooks and core Rust behavior
- Routing and config defaults
- Skills and skill registry
- Build, test, and release tasks

## Core Workflow

### Agents

1. Create or update `agents/<id>.md`.
2. Update `agents/agents.json` to match the metadata in the agent file.
3. Update catalog docs if the agent is new or materially changed.
4. Validate with `just validate-agents` (or `python3 scripts/validate_agents.py`).
5. Fix formatting or schema issues with `python3 scripts/fix_agents.py` when needed.

Read detailed requirements in:
- `docs/AGENT_GUIDE.md`
- `docs/AGENT_SPECIFICATION.md`
- `docs/AGENT_MAINTENANCE.md`

### Commands

1. Add or edit a TOML file under `commands/` or `commands/agent/`.
2. Keep prompts aligned with existing patterns in `commands/system-agent.toml` and `commands/agent/*.toml`.
3. If a new command affects agents or skills, update documentation that lists commands.

### Hooks

1. Create a module in `crates/core/src/hooks/`.
2. Register the module and re-export the hook in `crates/core/src/hooks/mod.rs`.
3. Add or update any hook configuration or defaults if required.
4. Test hooks with `cargo test --package bl1nk-core hooks` or `just test` as appropriate.

Use hook guidance in `docs/guidline-agents-system.md` and architecture notes in `docs/ARCHITECTURE.md`.

### Routing And Config

1. Update routing rules or defaults in `config/Config.toml` and any related Rust config structs.
2. Ensure priorities and keywords remain consistent with the intended routing behavior.
3. When changing routing, validate with targeted tests or a focused run command.

### Skills

1. Add or update skill directories in `skills/`.
2. If using the skill registry, update `skills/skills.json` with a new entry or edits.
3. Keep skill metadata consistent with existing tags and schema.

### Build And Test

Preferred commands:
- `just build`, `just run`, `just dev`
- `just test`, `just check`, `just fmt`, `just clippy`

## Reference Files

Use `references/workflows.md` for checklists, paths, and registry details.
