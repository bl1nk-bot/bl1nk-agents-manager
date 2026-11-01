# Specification: Discovery Engine (The Truth Provider)

## 1. Overview
The Discovery Engine is the "eyes" of the system. It performs a read-only scan of the user's environment to identify available AI CLIs (Gemini, Claude, etc.), Git integration, and Package Managers (npm, cargo). The goal is to provide an objective report of the machine's state to empower the system's logic and the user's decisions.

## 2. Functional Requirements
- **Resource Scanning:**
    - Detect AI CLIs: `gemini`, `claude`, `qwen`, `ollama`.
    - Detect Version Control: `git`.
    - Detect Package Managers: `npm`, `pnpm`, `yarn`, `cargo`.
- **Global Persistence:**
    - Store discovery results in the Global User Configuration directory (`~/.config/bl1nk-agents-manager/discovery.json`).
- **Reporting:**
    - Provide a unified API/struct for other modules to query the "Truth" of the local machine.
- **Refresh Logic:**
    - Allow manual and automated triggers to re-scan the environment.

## 3. Non-Functional Requirements
- **Speed:** Scanning must be asynchronous and non-blocking.
- **Accuracy:** Use standard `which` or `Command::new` checks to verify binary existence and version.
- **Philosophy:** Report facts only. Do not attempt to fix or install missing tools.

## 4. Acceptance Criteria
- A new module `src/system/discovery.rs` exists.
- Running the discovery logic produces a valid JSON file in the global config path.
- TUI/CLI can access this data to show tool availability.
