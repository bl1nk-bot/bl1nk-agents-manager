# Specification: Implement Terminal-Centric Visual Interface (TUI)

## 1. Overview
This track focuses on building the foundational Terminal-Centric Visual Interface (TUI) for the System Agents Manager. The goal is to provide a modern, interactive, keyboard-first interface within the terminal for users to inspect, list, and switch agents without needing the full desktop GUI.

## 2. Functional Requirements
-   **Agent Listing:** Display a scrollable list of available agents with their IDs and descriptions.
-   **Agent Inspection:** Allow users to select an agent to view detailed metadata (system prompt, version, capabilities).
-   **Keyboard Navigation:** Full support for arrow keys (Up/Down) to navigate lists and Enter to select/inspect.
-   **Command Execution:** Provide a way to "Switch" to the selected agent (generating the necessary export command).
-   **Exit Mechanism:** A clear way to exit the TUI (e.g., 'q' or Esc).

## 3. Non-Functional Requirements
-   **Performance:** The TUI should load instantly (<200ms) and respond immediately to user input.
-   **Compatibility:** Must render correctly in standard terminal emulators (xterm-256color support).
-   **Library:** specific Rust TUI library is not strictly enforced but `ratatui` (formerly `tui-rs`) is recommended for the Rust ecosystem.

## 4. User Experience
-   **Minimalist Design:** Clean borders, clear highlighting of the active selection.
-   **Information Density:** Show the list on the left (or top) and details on the right (or bottom) if space permits, otherwise use a master-detail view.

## 5. Out of Scope
-   Editing agent definitions (Read-only for this MVP).
-   Creating new agents via TUI.
-   Mouse support (Nice to have, but not required for MVP).
