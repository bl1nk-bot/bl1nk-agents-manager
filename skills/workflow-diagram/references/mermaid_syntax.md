# Mermaid Syntax Reference
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

## Core Diagram Types

### 1. State Diagrams

**Purpose**: Show system states and transitions

```mermaid
stateDiagram-v2
    [*] --> Active
    Active --> Inactive : timeout
    Inactive --> Active : activate
```

**Key Elements**:
- `[*]` - Start/end state
- `StateName` - State declaration
- `StateA --> StateB : label` - Transition with label
- `StateA --> StateB : event` - Event trigger

### 2. Flowcharts

**Purpose**: Show process flows and decision logic

```mermaid
graph TD
    A[Start] --> B{Decision}
    B -->|Yes| C[Action 1]
    B -->|No| D[Action 2]
    C --> E[End]
    D --> E
```

**Graph Directions**:
- `TD` - Top to Down (default)
- `LR` - Left to Right
- `BT` - Bottom to Top
- `RL` - Right to Left

**Node Shapes**:
- `A[Text]` - Rectangle
- `A(Text)` - Rounded rectangle
- `A{Text}` - Diamond
- `A((Text))` - Circle

### 3. Sequence Diagrams

**Purpose**: Show interaction between components over time

```mermaid
sequenceDiagram
    participant A as Actor
    participant S as System
    
    A->>S: Request
    S->>A: Response
    A->>S: Confirm
```

**Arrow Types**:
- `->` - Solid arrow
- `-->` - Dashed arrow
- `->>` - Open arrow
- `-->>` - Open dashed arrow

### 4. Gantt Charts

**Purpose**: Show project timeline and dependencies

```mermaid
gantt
    title Project Timeline
    dateFormat  YYYY-MM-DD
    section Phase 1
    Task A      :2024-01-01, 7d
    Task B      :2024-01-08, 5d
    section Phase 2
    Task C      :2024-01-15, 3d
```

## Advanced Features

### Subgraphs

```mermaid
graph TD
    subgraph "Phase 1"
        A[Step 1]
        B[Step 2]
    end
    subgraph "Phase 2"
        C[Step 3]
        D[Step 4]
    end
    A --> B
    B --> C
    C --> D
```

### Styling

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

### Comments and Notes

```mermaid
%% This is a comment
graph TD
    A[Start] --> B[End] %% Inline comment
```

## Kiro Workflow Specific Patterns

### State Machine for Development Phases

```mermaid
stateDiagram-v2
    [*] --> Requirements
    Requirements --> Review_Req : Complete
    Review_Req --> Design : Approved
    Review_Req --> Requirements : Changes
    Design --> Review_Design : Complete
    Review_Design --> Tasks : Approved
    Review_Design --> Design : Changes
    Tasks --> Review_Tasks : Complete
    Review_Tasks --> Execute : Approved
    Review_Tasks --> Tasks : Changes
    Execute --> [*] : Complete
```

### File Structure Visualization

```mermaid
graph TD
    A[.kiro/] --> B[specs/]
    B --> C[feature-name/]
    C --> D[requirements.md]
    C --> E[design.md]
    C --> F[tasks.md]
    
    style A fill:#e3f2fd
    style B fill:#e8f5e9
    style C fill:#fff3e0
    style D fill:#ffebee
    style E fill:#e3f2fd
    style F fill:#e8f5e9
```

## Common Syntax Rules

1. **Node IDs**: Use spaces or underscores, avoid special characters
2. **Labels**: Use quotes for labels with spaces: `--> B : "Label with spaces"`
3. **Escape Characters**: Use `#9829;` for special characters
4. **Line Breaks**: Use `<br/>` for text line breaks
5. **HTML**: Basic HTML supported in node text

## Integration Tips

- Use consistent naming across diagrams
- Apply color schemes systematically
- Keep diagrams readable (avoid overcrowding)
- Test syntax with validator script
- Export multiple formats for different use cases
