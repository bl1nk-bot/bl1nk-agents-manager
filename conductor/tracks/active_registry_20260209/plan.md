# Implementation Plan - Track: Active Registry

## Phase 1: Registry Refactoring
- [ ] Task: Update Agent Data Structures
    - [ ] Sub-task: Add `Availability` enum (Ready, MissingTool, Unknown) to `AgentConfig`
- [ ] Task: Implement Discovery Link
    - [ ] Sub-task: Modify `AgentRegistry::new` to accept discovery data
    - [ ] Sub-task: Implement requirement-to-fact matching logic
- [ ] Task: Conductor - User Manual Verification 'Phase 1: Registry Refactoring' (Protocol in workflow.md)

## Phase 2: Dynamic Selection Support
- [ ] Task: Implement Smart Sorting
    - [ ] Sub-task: Update `get_agents_by_priority` to consider availability
- [ ] Task: Conductor - User Manual Verification 'Phase 2: Dynamic Selection Support' (Protocol in workflow.md)
