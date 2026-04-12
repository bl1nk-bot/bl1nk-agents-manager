# Advanced Usage: Revision and Branching
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

## Revising Previous Thoughts

When a thought proves incorrect or incomplete, use revision to correct the reasoning chain:

```typescript
{
  thought: "Actually, the N+1 problem isn't the bottleneck—profiling shows the issue is missing indexes on join columns.",
  thoughtNumber: 5,
  totalThoughts: 7,
  isRevision: true,
  revisesThought: 2, // References thought #2
  nextThoughtNeeded: true
}
```

**When to revise**:
- New evidence contradicts earlier conclusions
- Assumptions prove incorrect
- Scope was misunderstood
- Need to correct factual errors

## Branching Into Alternatives

Explore different solution paths by branching from a specific thought:

```typescript
// Main path (thoughts 1-3)
{
  thought: "Could optimize with caching or database indexes.",
  thoughtNumber: 3,
  totalThoughts: 6,
  nextThoughtNeeded: true
}

// Branch A: Explore caching
{
  thought: "If we implement Redis caching, we'd need to handle cache invalidation.",
  thoughtNumber: 4,
  totalThoughts: 6,
  branchFromThought: 3,
  branchId: "caching-approach",
  nextThoughtNeeded: true
}

// Branch B: Explore indexing (alternative from thought 3)
{
  thought: "Adding composite index would avoid query overhead entirely.",
  thoughtNumber: 4,
  totalThoughts: 5,
  branchFromThought: 3,
  branchId: "indexing-approach",
  nextThoughtNeeded: true
}
```

**When to branch**:
- Multiple viable approaches exist
- Need to compare trade-offs
- Exploring contingencies
- Testing hypotheses in parallel

## Combining Revision and Branching

```typescript
// Original branch proves problematic
{
  thought: "The caching approach has too many edge cases for our timeline.",
  thoughtNumber: 6,
  totalThoughts: 8,
  branchId: "caching-approach",
  isRevision: true,
  revisesThought: 4,
  nextThoughtNeeded: true
}

// Return to indexing branch
{
  thought: "Returning to index optimization—this approach is more reliable.",
  thoughtNumber: 7,
  totalThoughts: 9,
  branchId: "indexing-approach",
  nextThoughtNeeded: true
}
```

## Dynamic Scope Adjustment

Freely adjust `totalThoughts` as understanding evolves:

```typescript
// Initial estimate
{ thoughtNumber: 1, totalThoughts: 5, ... }

// Complexity increases
{ thoughtNumber: 3, totalThoughts: 8, ... }

// Actually simpler than expected
{ thoughtNumber: 5, totalThoughts: 6, ... }
```

**Purpose**: Provide progress visibility, not strict planning. The estimate guides pacing but should adapt to reality.

## Session Management

Each reasoning session maintains its own context. The tool tracks:
- All thoughts in sequence
- Revision relationships
- Branch hierarchies
- Current state

You don't need to manually manage state—focus on expressing reasoning clearly.

## Best Practices

1. **Express uncertainty**: "This might be...", "Uncertain if...", "Need to verify..."
2. **Show reasoning**: Not just conclusions, but how you arrived there
3. **Revise freely**: Correcting course is expected and valuable
4. **Branch decisively**: When exploring alternatives, commit to exploring each fully
5. **Adjust scope**: Don't lock into initial estimates
6. **Maintain clarity**: Each thought should be self-contained enough to understand in isolation
