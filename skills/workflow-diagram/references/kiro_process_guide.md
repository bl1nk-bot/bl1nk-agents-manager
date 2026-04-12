# Kiro Process Guide
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

## Overview

Kiro is a structured development process that breaks down complex features into three distinct phases: Requirements, Design, and Tasks. This ensures thorough planning before execution and maintains traceability throughout development.

## Core Principles

### 1. Phase Separation
Each phase has a distinct purpose:
- **Requirements**: What and why
- **Design**: How and where
- **Tasks**: Who and when

### 2. Explicit Approval Gates
Progression between phases requires explicit approval, preventing premature implementation.

### 3. Document Dependencies
Later documents reference earlier ones, creating a dependency chain:
- `requirements.md` informs `design.md`
- `design.md` guides `tasks.md`
- `tasks.md` implements both requirements and design

## Phase Details

### Phase 1: Requirements (`requirements.md`)

**Purpose**: Define what needs to be built and why

**Key Elements**:
- Problem statement
- User stories or acceptance criteria
- Success metrics
- Constraints and assumptions
- Edge cases to consider

**Template Structure**:
```markdown
# Feature Name

## Problem Statement
[Clear description of the problem being solved]

## Acceptance Criteria
- [ ] Criteria 1
- [ ] Criteria 2
- [ ] Criteria 3

## Success Metrics
- Metric 1: [Definition and measurement]
- Metric 2: [Definition and measurement]

## Constraints
- Technical constraints
- Time/budget constraints
- Resource limitations

## Edge Cases
- Case 1: [Description and handling]
- Case 2: [Description and handling]
```

### Phase 2: Design (`design.md`)

**Purpose**: Define how the feature will be implemented

**Key Elements**:
- Architecture decisions
- API contracts
- Data models
- User interface considerations
- Integration points

**Template Structure**:
```markdown
# Feature Name - Design

## Architecture Overview
[High-level system design and major components]

## API Design
### Endpoints
- `GET /resource`: [Description]
- `POST /resource`: [Description]

### Data Models
```typescript
interface Resource {
  id: string;
  name: string;
  // other fields
}
```

## Database Schema
[Table structures, relationships, indexes]

## User Interface
[Key screens, interactions, navigation flow]

## Integration Points
[External services, APIs, or systems to integrate with]
```

### Phase 3: Tasks (`tasks.md`)

**Purpose**: Break down implementation into concrete tasks

**Key Elements**:
- Development tasks with acceptance criteria
- Order of implementation
- Dependencies between tasks
- Testing requirements

**Template Structure**:
```markdown
# Feature Name - Tasks

## Implementation Tasks

### Task 1: Setup Foundation
**Acceptance Criteria**:
- [ ] Project structure created
- [ ] Dependencies installed
- [ ] Basic configuration complete

**Dependencies**: None
**Estimated Time**: [Time estimate]

### Task 2: Core Implementation
**Acceptance Criteria**:
- [ ] Core functionality implemented
- [ ] Unit tests written
- [ ] Code reviewed

**Dependencies**: Task 1
**Estimated Time**: [Time estimate]

### Task 3: Integration & Testing
**Acceptance Criteria**:
- [ ] Integration tests pass
- [ ] User acceptance testing complete
- [ ] Documentation updated

**Dependencies**: Task 2
**Estimated Time**: [Time estimate]
```

## Workflow Entry Points

### New Feature Development
1. Start with `requirements.md`
2. Get approval before moving to design
3. Get approval before creating tasks
4. Execute tasks sequentially

### Feature Updates
1. Update affected documents
2. Maintain approval consistency
3. Re-execute affected tasks

### Bug Fixes
1. Document issue in requirements
2. Design solution approach
3. Create implementation tasks

## Best Practices

### Document Management
- Use kebab-case for feature directories
- Keep documents under version control
- Link documents with cross-references
- Maintain change logs

### Approval Process
- Require explicit "approved" confirmation
- Document feedback and changes
- Use review comments for traceability
- Store approval decisions

### Task Execution
- Work sequentially through task list
- Mark completion criteria
- Update blockers immediately
- Communicate progress regularly

## Integration with Development Tools

### Git Integration
```
.kiro/specs/feature-name/
├── requirements.md
├── design.md
├── tasks.md
└── .gitkeep
```

### Project Management Integration
- Create tickets for each task
- Link tickets to specification documents
- Track progress against acceptance criteria
- Update estimates based on actual work

### Code Review Integration
- Reference design during review
- Verify requirements coverage
- Check task completion status
- Update documents based on review feedback

## Common Patterns

### Web Development Features
- Requirements: User stories, UI requirements
- Design: Component architecture, API contracts
- Tasks: Frontend, backend, testing tasks

### Data Processing Features
- Requirements: Data sources, transformation rules
- Design: Pipeline architecture, storage schema
- Tasks: ETL implementation, validation tasks

### Integration Features
- Requirements: External system requirements
- Design: Integration patterns, data contracts
- Tasks: API implementation, error handling

This process ensures thorough planning, clear communication, and successful feature delivery while maintaining documentation for future reference.
