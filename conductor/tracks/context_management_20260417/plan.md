# Implementation Plan: Context Management

## Phase 1: Context Data Structures and Storage Trait [checkpoint: 916bca0]

- [x] Task: Write failing tests for context data structures (Conversation, Workspace, Secrets).
- [x] Task: Implement core data structures and `ContextStore` trait to abstract storage mechanisms.
- [x] Task: Conductor - User Manual Verification 'Phase 1: Context Data Structures and Storage Trait' (Protocol in workflow.md)

## Phase 2: JSON/File-based Storage Implementation [checkpoint: 916bca0]

- [x] Task: Write failing tests for saving/loading context using JSON files.
- [x] Task: Implement JSON-based `ContextStore` for JSON file persistence.
- [x] Task: Write failing tests for handling missing or corrupted context files.
- [x] Task: Implement robust async error handling for file I/O.
- [x] Task: Conductor - User Manual Verification 'Phase 2: JSON/File-based Storage Implementation' (Protocol in workflow.md)

## Phase 3: Context Compaction & Size Management [checkpoint: pending]

> **3 Features ที่เพิ่ม (from AI Coding Tools comparison):**
> 1. Token Budget Management - limit + auto-select important messages
> 2. Tool Use History Tracking - track tool calls across sessions
> 3. Context Offload to File - archive old messages

- [ ] Task: Write failing tests for context compaction logic (e.g., retaining top 20%).
- [ ] Task: Implement sliding window or compaction logic to safely manage token limits.
- [ ] Task: Implement token budget management (configurable limits, auto-eviction).
- [ ] Task: Write failing tests for offloading overflow context to readable files.
- [ ] Task: Implement file-based context offloading.
- [ ] Task: Implement tool use history tracking (store/cached results).
- [ ] Task: Conductor - User Manual Verification 'Phase 3: Context Compaction & Size Management' (Protocol in workflow.md)

## Phase 4: Environment & Secrets Handling [checkpoint: pending]

- [ ] Task: Write failing tests for securely loading, masking, and injecting secrets.
- [ ] Task: Implement secure handling of environment variables within the context system.
- [ ] Task: Conductor - User Manual Verification 'Phase 4: Environment & Secrets Handling' (Protocol in workflow.md)

## Phase 5: Integration with Orchestrator [checkpoint: pending]

- [ ] Task: Write integration tests for context injection into the agent execution loop.
- [ ] Task: Integrate context loading and updating into `src/agents/executor.rs` and the MCP gateway.
- [ ] Task: Conductor - User Manual Verification 'Phase 5: Integration with Orchestrator' (Protocol in workflow.md)
