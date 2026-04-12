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
name: sequential-thinking
description: Use when complex problems require systematic step-by-step reasoning with
  ability to revise thoughts, branch into alternative approaches, or dynamically adjust
  scope. Ideal for multi-stage analysis, design planning, problem decomposition, or
  tasks with initially unclear scope.
license: MIT
metadata:
  version: 1.0.0
  quality: FAIR
  last_updated: 2025-12-28 07:28:22.124750
---



























# Sequential Thinking

Enables structured problem-solving through iterative reasoning with revision and branching capabilities.

## Core Capabilities

- **Iterative reasoning**: Break complex problems into sequential thought steps
- **Dynamic scope**: Adjust total thought count as understanding evolves
- **Revision tracking**: Reconsider and modify previous conclusions
- **Branch exploration**: Explore alternative reasoning paths from any point
- **Maintained context**: Keep track of reasoning chain throughout analysis

## When to Use

Use `mcp__reasoning__sequentialthinking` when:
- Problem requires multiple interconnected reasoning steps
- Initial scope or approach is uncertain
- Need to filter through complexity to find core issues
- May need to backtrack or revise earlier conclusions
- Want to explore alternative solution paths

**Don't use for**: Simple queries, direct facts, or single-step tasks.

## Basic Usage

The MCP tool `mcp__reasoning__sequentialthinking` accepts these parameters:

### Required Parameters

- `thought` (string): Current reasoning step
- `nextThoughtNeeded` (boolean): Whether more reasoning is needed
- `thoughtNumber` (integer): Current step number (starts at 1)
- `totalThoughts` (integer): Estimated total steps needed

### Optional Parameters

- `isRevision` (boolean): Indicates this revises previous thinking
- `revisesThought` (integer): Which thought number is being reconsidered
- `branchFromThought` (integer): Thought number to branch from
- `branchId` (string): Identifier for this reasoning branch

## Workflow Pattern

```
1. Start with initial thought (thoughtNumber: 1)
2. For each step:
   - Express current reasoning in `thought`
   - Estimate remaining work via `totalThoughts` (adjust dynamically)
   - Set `nextThoughtNeeded: true` to continue
3. When reaching conclusion, set `nextThoughtNeeded: false`
```

## Simple Example

```typescript
// First thought
{
  thought: "Problem involves optimizing database queries. Need to identify bottlenecks first.",
  thoughtNumber: 1,
  totalThoughts: 5,
  nextThoughtNeeded: true
}

// Second thought
{
  thought: "Analyzing query patterns reveals N+1 problem in user fetches.",
  thoughtNumber: 2,
  totalThoughts: 6, // Adjusted scope
  nextThoughtNeeded: true
}

// ... continue until done
```

## Advanced Features

For revision patterns, branching strategies, and complex workflows, see:
- [Advanced Usage ](references/advanced.md) - Revision and branching patterns
- [Examples ](references/examples.md) - Real-world use cases

## Tips

- Start with rough estimate for `totalThoughts`, refine as you progress
- Use revision when assumptions prove incorrect
- Branch when multiple approaches seem viable
- Express uncertainty explicitly in thoughts
- Adjust scope freely - accuracy matters less than progress visibility
