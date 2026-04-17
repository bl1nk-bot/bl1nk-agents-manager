# Specification: Context Management

## 1. Overview

The goal of this track is to implement a comprehensive Context Management system for the BL1NK Agents Manager. This feature will be ported from an existing TypeScript implementation into Rust. The system will handle various types of context (conversation history, workspace/file context, and environment variables/secrets) to ensure agents have the necessary information to perform their tasks effectively.

## 2. Scope

### 2.1 In Scope

- **Multi-faceted Context Handling:**
  - Manage conversation history (messages and turns).
  - Manage workspace and file-specific context.
  - Securely handle environment variables and secrets.
- **Storage Mechanism:**
  - Initial implementation using JSON/File-based storage (e.g., within `.omg/state`).
  - Prepare architecture for a smooth transition to SQLite database storage in subsequent iterations.
- **Context Size Management:**
  - Implement a compaction strategy (e.g., retaining 20% of recent context and compacting the rest) or utilize file-based context generation to handle token limits.
- **Porting:** Translate existing TypeScript logic for context management into idiomatic, memory-safe Rust.

### 2.2 Out of Scope

- Full migration to SQLite in this specific track (to be handled as a fast-follow).
- Modifying the underlying LLM models or their native context windows.

## 3. Functional Requirements

- **FR1:** The system MUST provide an interface to store, retrieve, and update conversation history.
- **FR2:** The system MUST provide an interface to load and inject workspace/file context into the agent's prompt or session.
- **FR3:** The system MUST securely manage and inject environment variables required by agents without exposing them in plaintext logs.
- **FR4:** The system MUST persist context data to the local filesystem using JSON format initially.
- **FR5:** The system MUST implement a mechanism to prevent context overflow, either by summarizing/compacting older messages (keeping ~20%) or offloading context to readable files.

## 4. Non-Functional Requirements

- **Performance:** File I/O operations for context should be asynchronous to prevent blocking the orchestrator.
- **Maintainability:** The codebase must follow the established Rust style guide (`conductor/code_styleguides/rust.md`).
- **Extensibility:** The storage interface must be abstracted (e.g., via traits) to easily swap JSON storage with SQLite in the future.

## 5. Acceptance Criteria

- [ ] A Rust module for context management is created, porting the necessary TS logic.
- [ ] Conversation, file, and environment contexts can be saved to and loaded from JSON files.
- [ ] A context compaction or file-offloading mechanism is implemented and tested.
- [ ] Unit tests achieve >90% coverage for the new context management module.
- [ ] The system handles missing context files gracefully without crashing.
