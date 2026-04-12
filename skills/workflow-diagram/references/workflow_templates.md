# Common Workflow Templates
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

## Kiro Development Workflow

### Basic State Machine
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

### Phase Progression
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

## Software Development Workflows

### Agile Sprint Workflow
```mermaid
stateDiagram-v2
    [*] --> Planning : Sprint Start
    Planning --> Development : Story Points Set
    Development --> Review : Code Complete
    Review --> Testing : Review Passed
    Testing --> Deployment : Tests Passed
    Deployment --> Retrospective : Deployed
    Retrospective --> Planning : Sprint End
    
    Review --> Development : Changes Required
    Testing --> Development : Bugs Found
```

### CI/CD Pipeline
```mermaid
graph TD
    A[Push Code] --> B[CI Build]
    B --> C{Tests Pass?}
    C -->|Yes| D[Deploy to Staging]
    C -->|No| E[Notify Developer]
    D --> F[Integration Tests]
    F --> G{Deploy to Prod?}
    G -->|Yes| H[Production Deploy]
    G -->|No| I[Manual Review]
    E --> J[Fix Issues]
    H --> K[Monitor]
    I --> G
    J --> B
    
    style C fill:#ff9800
    style G fill:#2196f3
    style K fill:#4caf50
```

## Task Management Workflows

### Task Execution Flow
```mermaid
graph TD
    A[Receive Task] --> B[Analyze Requirements]
    B --> C[Break Down Sub-tasks]
    C --> D{Complex?}
    D -->|Simple| E[Execute Directly]
    D -->|Complex| F[Create Sub-tasks]
    F --> G[Execute in Order]
    G --> H{More Sub-tasks?}
    H -->|Yes| I[Next Sub-task]
    H -->|No| J[Review & Complete]
    E --> J
    I --> G
    J --> K[Mark Complete]
```

### Bug Fix Workflow
```mermaid
stateDiagram-v2
    [*] --> Reported : Bug Found
    Reported --> Triage : Initial Report
    Triage --> Investigate : Triage Complete
    Investigate --> Fix : Root Cause Found
    Fix --> Test : Fix Implemented
    Test --> Verify : Unit Tests Pass
    Verify --> Deploy : QA Approved
    Deploy --> Monitor : Deployed to Production
    Monitor --> Closed : Issue Resolved
    
    Triage --> Reported : Invalid/Duplicate
    Test --> Fix : Tests Fail
    Verify --> Test : QA Rejected
    Monitor --> Fix : Regression Detected
```

## Code Review Workflows

### Pull Request Process
```mermaid
sequenceDiagram
    participant Dev as Developer
    participant CR as Code Reviewer
    participant CI as CI System
    participant Repo as Repository
    
    Dev->>Repo: Create Branch
    Dev->>Repo: Push Changes
    Dev->>Repo: Create PR
    CI->>Repo: Run Tests
    CI->>Dev: Test Results
    CR->>Repo: Review PR
    CR->>Dev: Review Comments
    Dev->>Repo: Update PR
    CR->>Repo: Approve PR
    Repo->>Repo: Merge to Main
```

## Approval Gate Patterns

### Multi-level Approval
```mermaid
graph TD
    A[Submit Work] --> B[Peer Review]
    B --> C{Peer Approved?}
    C -->|No| D[Address Feedback]
    C -->|Yes| E[Lead Review]
    D --> B
    E --> F{Lead Approved?}
    F -->|No| G[Address Feedback]
    F -->|Yes| H[Manager Approval]
    G --> E
    H --> I{Manager Approved?}
    I -->|No| J[Escalate/Rework]
    I -->|Yes| K[Approved]
    
    style C fill:#ff9800
    style F fill:#2196f3
    style I fill:#4caf50
    style K fill:#4caf50
```

## Documentation Workflows

### Document Lifecycle
```mermaid
stateDiagram-v2
    [*] --> Draft : Initial Creation
    Draft --> Review : Content Complete
    Review --> Revision : Feedback Received
    Revision --> Review : Changes Made
    Review --> Approved : No Feedback
    Approved --> Published : Release Ready
    Published --> Archived : Outdated
    
    Approved --> Revision : Major Changes
    Published --> Review : Updates Needed
```

## Release Management

### Feature Release Pipeline
```mermaid
graph LR
    A[Feature Complete] --> B[QA Testing]
    B --> C{QA Passed?}
    C -->|No| D[Fix Issues]
    C -->|Yes| E[Staging Deploy]
    D --> A
    E --> F[UAT Testing]
    F --> G{UAT Passed?}
    G -->|No| H[Address Feedback]
    G -->|Yes| I[Production Deploy]
    H --> E
    
    style C fill:#f44336
    style G fill:#ff9800
    style I fill:#4caf50
```

## Usage Instructions

1. **Select Template**: Choose the template closest to your workflow
2. **Customize**: Modify nodes, transitions, and labels
3. **Apply Styling**: Add colors and themes for clarity
4. **Validate**: Use syntax validator to check correctness
5. **Export**: Generate HTML or SVG for sharing

## Customization Tips

- **Rename nodes** to match your specific terms
- **Adjust transitions** for your process steps
- **Add decision points** where choices occur
- **Include feedback loops** for iterative processes
- **Apply consistent colors** for phase identification
