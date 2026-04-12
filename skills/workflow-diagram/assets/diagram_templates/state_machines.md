# Kiro Workflow Templates
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

## Complete Kiro State Machine

### Full Development Lifecycle
```mermaid
stateDiagram-v2
  [*] --> Requirements : Initial Creation

  Requirements : Write Requirements
  Design : Write Design
  Tasks : Write Tasks
  Execute : Execute Task

  Requirements --> ReviewReq : Complete Requirements
  ReviewReq --> Requirements : Feedback/Changes Requested
  ReviewReq --> Design : Explicit Approval

  Design --> ReviewDesign : Complete Design
  ReviewDesign --> Design : Feedback/Changes Requested
  ReviewDesign --> Tasks : Explicit Approval

  Tasks --> ReviewTasks : Complete Tasks
  ReviewTasks --> Tasks : Feedback/Changes Requested
  ReviewTasks --> Execute : Explicit Approval

  Execute --> [*] : Complete
```

## Entry Point Specific Templates

### Requirements Phase
```mermaid
stateDiagram-v2
  [*] --> Gathering : Start
  Gathering --> Writing : Research Complete
  Writing --> Review : Draft Complete
  Review --> Revising : Feedback Received
  Revising --> Review : Changes Made
  Review --> Approved : All Feedback Addressed
  Approved --> [*] : Ready for Design
```

### Design Phase
```mermaid
stateDiagram-v2
  [*] --> Architecture : Requirements Received
  Architecture --> Detailed : High-Level Complete
  Detailed --> Prototyping : Detailed Design Done
  Prototyping --> Validation : Prototype Ready
  Validation --> Revising : Issues Found
  Revising --> Validation : Changes Made
  Validation --> Approved : Design Validated
  Approved --> [*] : Ready for Tasks
```

### Task Execution
```mermaid
stateDiagram-v2
  [*] --> Planning : Design Received
  Planning --> Implementation : Tasks Created
  Implementation --> Testing : Code Complete
  Testing --> Debugging : Tests Fail
  Testing --> Review : Tests Pass
  Debugging --> Implementation : Fixes Made
  Review --> Deployment : Code Approved
  Deployment --> [*] : Feature Complete
```

## Approval Gate Templates

### Multi-Stage Approval
```mermaid
graph TD
    A[Submit Work] --> B[Self Review]
    B --> C[Peer Review]
    C --> D{Peer Approved?}
    D -->|No| E[Address Feedback]
    D -->|Yes| F[Lead Review]
    E --> B
    F --> G{Lead Approved?}
    G -->|No| H[Address Feedback]
    G -->|Yes| I[Final Approval]
    H --> F
    I --> J[Proceed to Next Phase]
    
    style D fill:#ff9800
    style G fill:#2196f3
    style I fill:#4caf50
```

### Simple Approval
```mermaid
graph LR
    A[Work Complete] --> B{Approved?}
    B -->|No| C[Revise]
    B -->|Yes| D[Next Phase]
    C --> A
    
    style B fill:#ff9800
    style D fill:#4caf50
```
