# Kiro Sequence Diagram Templates
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

## Development Process Sequences

### Standard Development Flow
```mermaid
sequenceDiagram
    participant User as Product Owner
    participant Dev as Developer
    participant Reviewer as Code Reviewer
    participant CI as CI/CD System
    participant Deploy as Production
    
    User->>Dev: Feature Request
    Dev->>User: Questions/Clarifications
    User->>Dev: Requirements Confirmation
    Dev->>Dev: Implementation
    Dev->>Reviewer: Pull Request
    Reviewer->>Dev: Review Feedback
    Dev->>Reviewer: Updated PR
    Reviewer->>CI: Approve PR
    CI->>Deploy: Deploy Feature
    Deploy->>User: Feature Available
    User->>Deploy: Acceptance Testing
```

### Agile Sprint Sequence
```mermaid
sequenceDiagram
    participant PO as Product Owner
    participant Team as Dev Team
    participant SM as Scrum Master
    participant QA as QA Team
    
    PO->>Team: Sprint Planning
    Team->>PO: Story Points
    PO->>Team: Confirm Sprint
    Team->>Team: Daily Standups
    Team->>QA: Feature for Testing
    QA->>Team: Bug Reports
    Team->>QA: Bug Fixes
    Team->>PO: Sprint Review
    PO->>Team: Sprint Retrospective
```

## Review and Approval Sequences

### Code Review Process
```mermaid
sequenceDiagram
    participant Dev as Developer
    participant Auto as Automated Checks
    participant Reviewer as Code Reviewer
    participant Repo as Repository
    
    Dev->>Repo: Push Branch
    Auto->>Dev: Build Results
    Auto->>Dev: Test Results
    Dev->>Repo: Create PR
    Auto->>Repo: Run Checks on PR
    Reviewer->>Repo: Review PR
    Reviewer->>Dev: Review Comments
    Dev->>Repo: Address Feedback
    Reviewer->>Repo: Approve PR
    Repo->>Repo: Merge to Main
```

### Multi-Stage Approval
```mermaid
sequenceDiagram
    participant Dev as Developer
    participant Peer as Peer Reviewer
    participant Lead as Team Lead
    participant Manager as Engineering Manager
    participant System as Project System
    
    Dev->>Peer: Submit for Review
    Peer->>Dev: Feedback/Questions
    Dev->>Peer: Updated Work
    Peer->>Lead: Recommend Approval
    Lead->>Dev: Additional Feedback
    Dev->>Lead: Address Feedback
    Lead->>Manager: Recommend Approval
    Manager->>System: Final Approval
    System->>Dev: Work Approved
```

## Deployment Sequences

### CI/CD Pipeline
```mermaid
sequenceDiagram
    participant Dev as Developer
    participant Git as Git Repository
    participant CI as CI Server
    participant Staging as Staging Environment
    participant QA as QA Team
    participant Prod as Production
    
    Dev->>Git: Push Code
    Git->>CI: Trigger Build
    CI->>CI: Run Tests
    CI->>CI: Build Artifacts
    CI->>Staging: Deploy to Staging
    Staging->>QA: Ready for Testing
    QA->>Staging: Execute Tests
    QA->>CI: Test Results
    CI->>Prod: Deploy to Production
    Prod->>QA: Production Ready
    QA->>Prod: Final Verification
```

### Database Migration
```mermaid
sequenceDiagram
    participant Dev as Developer
    participant Local as Local DB
    participant Migrator as Migration Tool
    participant Staging as Staging DB
    participant Prod as Production DB
    
    Dev->>Local: Create Migration
    Dev->>Migrator: Migration Script
    Migrator->>Local: Test Migration
    Local->>Dev: Migration Success
    Dev->>Staging: Deploy Migration
    Migrator->>Staging: Run Migration
    Staging->>Dev: Migration Complete
    Dev->>Migrator: Approve for Production
    Migrator->>Prod: Deploy Migration
    Prod->>Dev: Production Migration Complete
```

## Communication Sequences

### Requirements Gathering
```mermaid
sequenceDiagram
    participant PO as Product Owner
    participant Dev as Developer
    participant Designer as UX Designer
    participant QA as QA Engineer
    
    PO->>Designer: Feature Requirements
    Designer->>PO: Design Questions
    Designer->>Dev: Design Specifications
    Dev->>Designer: Technical Questions
    Dev->>QA: Testing Requirements
    QA->>Dev: Test Case Review
    Dev->>PO: Implementation Plan
    PO->>Dev: Requirements Confirmation
```

### Bug Report Handling
```mermaid
sequenceDiagram
    participant User as End User
    participant Support as Support Team
    participant Dev as Developer
    participant QA as QA Team
    participant Prod as Production
    
    User->>Support: Bug Report
    Support->>QA: Reproduce Issue
    QA->>Prod: Investigation
    QA->>Dev: Bug Report
    Dev->>QA: Root Cause Analysis
    Dev->>Dev: Fix Implementation
    Dev->>QA: Fix for Testing
    QA->>Prod: Deploy Fix
    QA->>Support: Fix Verified
    Support->>User: Issue Resolved
```
