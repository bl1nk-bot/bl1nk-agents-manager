# Product Guidelines

## Tone and Voice
-   **Professional & Concise:** Interactions should be direct and efficient, respecting the user's time. Avoid unnecessary pleasantries or conversational filler.
-   **Technical & Precise:** Use exact industry terminology. Assume the user is a technical expert (engineer/developer) and values accuracy over simplification.
-   **Persona Flexibility:** While the core system is professional, specific agents (e.g., entertainment personas) may deviate from this rule as defined by their individual system prompts.

## User Interface (TUI/GUI) Design Principles
-   **Minimalism:** Prioritize clean layouts and reducing visual clutter to maintain focus.
-   **Information Density:** intelligently utilize screen real estate to present relevant data without overwhelming the user.
-   **Keyboard-First:** Ensure every action is performable via keyboard shortcuts. The mouse should be optional.
-   **Accessibility:** Adhere to high contrast standards and ensure compatibility with screen readers to support all developers.

## Engineering & Architecture Standards
-   **Modularity:** Maintain strict separation of concerns. Agents, Hooks, and Core logic must be decoupled to allow independent evolution.
-   **Safety:** Prioritize Rust for core components to leverage its memory safety guarantees.
-   **Async-First:** Build on the Tokio runtime. All I/O operations must be non-blocking to ensure high performance and responsiveness.
-   **Documentation:** Every public API, agent definition, and hook must have comprehensive documentation. "Code as documentation" is not sufficient.

## Error Handling & Reliability
-   **Graceful Degradation:** The system must remain robust. Failure of a peripheral component (e.g., a specific hook) should not crash the entire application.
-   **Explicit Error Messages:** Errors must be descriptive and actionable, providing the user with a clear path to resolution.
-   **Automated Recovery:** Leverage hooks to implement self-healing capabilities, automatically recovering from transient issues like context window exhaustion.
-   **Fail-Fast:** For critical data integrity operations, the system should halt immediately upon error to prevent corruption.

## Contribution Guidelines
-   **Standardized Metadata:** All agents and hooks must rigidly adhere to the defined JSON schema.
-   **Review Process:** All contributions are subject to automated validation checks and mandatory peer review.
-   **Backwards Compatibility:** Changes must not break existing user workflows or agent definitions.
-   **License Compatibility:** All contributions must be compatible with the project's open-source license (MIT/Apache 2.0).
