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
name: workflow-diagram
description: Create and manage Kiro workflow diagrams with Mermaid syntax. Use when Claude needs to visualize software development workflows, state machines, phase progressions, approval gates, or task execution flows. Supports creating state diagrams, flowcharts, sequence diagrams, and organizing workflow documentation with proper file structure and document dependencies.
---

# Kiro Workflow Diagrams

## Overview

Create comprehensive workflow diagrams using Mermaid syntax to visualize software development processes, state machines, and task execution flows.

## Quick Start

Create a basic state diagram:
```mermaid
stateDiagram-v2
  [*] --> Requirements : Initial Creation
  Requirements --> Design : Complete
  Design --> Tasks : Complete
  Tasks --> [*] : Complete
```

## Core Workflow Patterns

### 1. State Machine Diagrams

Use for complete workflow visualization with states and transitions:

```mermaid
stateDiagram-v2
  [*] --> Requirements : Initial Creation

  Requirements : Write Requirements
  Design : Write Design
  Tasks : Write Tasks

  Requirements --> ReviewReq : Complete Requirements
  ReviewReq --> Requirements : Feedback/Changes Requested
  ReviewReq --> Design : Explicit Approval

  Design --> ReviewDesign : Complete Design
  ReviewDesign --> Design : Feedback/Changes Requested
  ReviewDesign --> Tasks : Explicit Approval

  Tasks --> ReviewTasks : Complete Tasks
  ReviewTasks --> Tasks : Feedback/Changes Requested
  ReviewTasks --> [*] : Explicit Approval
```

### 2. Phase Progression Flowcharts

Use for linear phase progression with decision points:

```mermaid
graph LR
    A[Idea] --> B[Requirements]
    B --> C{Approved?}
    C -->|No| B
    C -->|Yes| D[Design]
    D --> E{Approved?}
    E -->|No| D
    E -->|Yes| F[Tasks]
    F --> G{Approved?}
    G -->|No| F
    G -->|Yes| H[Execute]
    H --> I[Complete]
```

### 3. Entry Point Diagrams

Use for showing multiple workflow entry points:

```mermaid
graph TD
    A[User Request] --> B{What Phase?}
    B -->|New Feature| C[Start Requirements]
    B -->|Update Requirements| D[Edit Requirements]
    B -->|Create Design| E[Start Design]
    B -->|Update Design| F[Edit Design]
    B -->|Generate Tasks| G[Create Tasks]
    B -->|Update Tasks| H[Edit Tasks]
    B -->|Execute Task| I[Run Task]
```

### 4. Sequence Diagrams

Use for approval gate interactions:

```mermaid
sequenceDiagram
    participant U as User
    participant K as Kiro
    participant D as Document

    K->>D: Create/Update Document
    K->>U: "Does this look good?"
    U->>K: Feedback
    K->>D: Update Based on Feedback
    K->>U: "Does this look good?"
    U->>K: "Yes, approved"
    K->>K: Proceed to Next Phase
```

### 5. Document Dependencies

Use for showing relationships between specification files:

```mermaid
graph TD
    A[requirements.md] -->|Informs| B[design.md]
    B -->|Guides| C[tasks.md]
    C -->|References| A
    C -->|Implements| B

    style A fill:#ffebee
    style B fill:#e3f2fd
    style C fill:#e8f5e9
```

## File Structure Templates

### Kiro Specification Structure

```
.kiro/
└── specs/
    └── {feature-name}/    # kebab-case
        ├── requirements.md  # Phase 1
        ├── design.md        # Phase 2
        └── tasks.md         # Phase 3
```

### Diagram Organization

```
docs/
├── workflows/
│   ├── main-workflow.md      # Main state machine
│   ├── phase-progression.md  # Linear progression
│   ├── entry-points.md       # Entry point flows
│   └── task-execution.md     # Task execution flows
├── dependencies/
│   ├── file-structure.md     # Directory structure
│   └── document-deps.md      # Document relationships
└── approvals/
    └── approval-gates.md     # Approval sequences
```

## Task Execution Flow Patterns

### Complex Task Execution

```mermaid
graph TD
    A[User: Execute Task X] --> B[Read Spec Files]
    B --> C[requirements.md]
    B --> D[design.md]
    B --> E[tasks.md]

    C --> F[Understand Context]
    D --> F
    E --> F

    F --> G[Identify Task]
    G --> H{Has Sub-tasks?}
    H -->|Yes| I[Execute Sub-task First]
    H -->|No| J[Implement Task]

    I --> K[Complete Sub-task]
    K --> L{More Sub-tasks?}
    L -->|Yes| I
    L -->|No| J

    J --> M[Stop - Await Review]
    M --> N[User Approval]
    N --> O{More Tasks?}
    O -->|Yes| A
    O -->|No| P[Feature Complete]
```

## Styling and Customization

### Color Coding

Use consistent color schemes for different phases:
- **Requirements**: Red/pink tones (`#ffebee`, `#ffcdd2`)
- **Design**: Blue tones (`#e3f2fd`, `#bbdefb`)
- **Tasks**: Green tones (`#e8f5e9`, `#c8e6c9`)
- **Execution**: Purple/gray tones (`#f3e5f5`, `#e0e0e0`)

### Node Styling

```mermaid
graph TD
    A[Start] --> B[Process]
    B --> C{Decision}
    C -->|Yes| D[Success]
    C -->|No| E[Failure]
    
    style A fill:#4caf50,color:#fff
    style B fill:#2196f3,color:#fff
    style C fill:#ff9800,color:#fff
    style D fill:#4caf50,color:#fff
    style E fill:#f44336,color:#fff
```

## Best Practices

### Diagram Design Principles

1. **Keep it simple** - Avoid overcrowding with too many nodes
2. **Use consistent naming** - Maintain same terminology across diagrams
3. **Color code phases** - Use colors to distinguish workflow phases
4. **Add clear labels** - Label transitions and decision points clearly
5. **Organize hierarchically** - Place related elements together

### Documentation Integration

1. **Link to specs** - Reference actual specification files
2. **Include examples** - Show concrete workflow examples
3. **Maintain consistency** - Use same terminology as codebase
4. **Version control** - Track diagram changes with workflow updates

### Common Patterns

- **Feedback loops** - Show iteration and review cycles
- **Parallel processes** - Use subgraphs for concurrent workflows
- **Entry/exit points** - Clearly mark workflow boundaries
- **Decision gates** - Highlight approval and review points

## Resources

### scripts/
Python scripts for diagram generation and validation:
- `generate_diagrams.py` - Convert markdown to HTML diagrams
- `validate_syntax.py` - Check Mermaid syntax validity
- `export_svg.py` - Export diagrams to SVG format

### references/
- `mermaid_syntax.md` - Complete Mermaid syntax reference
- `workflow_templates.md` - Common workflow pattern templates
- `kiro_process_guide.md` - Detailed Kiro process documentation

### assets/
- `diagram_templates/` - Reusable diagram templates
- `style_themes/` - CSS themes for diagram styling
- `example_workflows/` - Complete workflow examples