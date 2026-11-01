# Implementation Plan - Track: Implement Terminal-Centric Visual Interface (TUI)

## Phase 1: Environment Setup & Foundation
- [x] Task: Set up Rust project structure for TUI [5dfcd34]
    - [x] Sub-task: Initialize new Rust crate `crates/tui` or add to `crates/core/src/bin/tui` (depending on preference, likely a new crate or bin)
    - [x] Sub-task: Update `Cargo.toml` with dependencies (`ratatui`, `crossterm`, `tokio`, `anyhow`)
    - [x] Sub-task: Create basic `main.rs` that initializes and restores the terminal
- [x] Task: Implement basic Event Loop [08368c9]
    - [x] Sub-task: Create `Event` enum (Key, Tick)
    - [x] Sub-task: Implement event handler loop using `crossterm` event polling
- [x] Task: Conductor - User Manual Verification 'Phase 1: Environment Setup & Foundation' (Protocol in workflow.md) [checkpoint: 29e556d]

## Phase 2: Agent Listing & Inspection
- [x] Task: Implement Agent Data Model [1d37731]
    - [x] Sub-task: Define `Agent` struct matching the existing agent definition
    - [x] Sub-task: Implement function to load agents from `agents/agents.json`
- [x] Task: Implement Agent List UI [d9959e9]
    - [x] Sub-task: Create `ui/list.rs` component to render the list of agents
    - [x] Sub-task: Implement state management for the selected index
- [x] Task: Implement Agent Detail UI [73f7d9b]
    - [x] Sub-task: Create `ui/detail.rs` component to render agent metadata (description, system prompt path)
    - [x] Sub-task: Integrate Detail view with the main layout (Split view)
- [x] Task: Conductor - User Manual Verification 'Phase 2: Agent Listing & Inspection' (Protocol in workflow.md) [checkpoint: 2dd4a26]

## Phase 3: Interaction & Core Integration
- [x] Task: Refactor to use `bl1nk-core` [1951ca6]
    - [ ] Sub-task: Add `bl1nk-core` dependency to `crates/tui/Cargo.toml`
    - [ ] Sub-task: Replace `models.rs` usage with `bl1nk_core::agents::AgentRegistry`
    - [ ] Sub-task: Fix `main.rs` and `ui/` components to match `bl1nk-core` types
- [x] Task: Implement "Switch" Command [da2df2d]
    - [ ] Sub-task: Implement logic to generate the export command for the selected agent
    - [ ] Sub-task: Display the command via a popup or copy to clipboard
- [~] Task: Conductor - User Manual Verification 'Phase 3: Interaction & Core Integration' (Protocol in workflow.md)