# Initial Concept
To create the MVP of this project, I suggest the following track:
- Build the core functionality for the tip calculator with a basic calculator and built-in tip percentages.

# Product Definition

## Vision
The System Agents Manager for Gemini CLI aims to transform the generic AI assistant experience into a highly specialized, reliable, and extensible ecosystem. By providing a standardized framework for managing AI personas, the project empowers users to leverage expert knowledge for specific tasks while automating complex workflows through a robust hooks system. The philosophy centers on the belief that specialized agents, transparent in their design and bolstered by built-in reliability mechanisms, offer superior performance and user experience compared to generalist models.

## Target Audience
- **Gemini CLI Power Users:** Individuals seeking to optimize their workflow with specialized personas tailored to specific engineering, research, or creative tasks.
- **Developers:** Contributors looking to extend Gemini CLI's capabilities by building custom agents, hooks, and integrations.
- **Teams:** Groups requiring shared, standardized agent definitions and workflows to ensure consistency and efficiency across collaborative projects.

## Core Goals
1.  **Standardized Management:** Establish a unified protocol and CLI interface for listing, inspecting, switching, and managing AI agents.
2.  **Advanced Automation:** Enable sophisticated, context-aware behaviors through a comprehensive hooks system that handles context injection, monitoring, and error recovery.
3.  **Extensibility:** Foster a community-driven ecosystem where users can easily create, share, and integrate custom agents and skills.
4.  **Resource Awareness:** Implement an intelligent core that is aware of local machine resources (CLIs, tools, costs) to empower user decision-making and optimize agent selection.

## Key Features
-   **Interactive CLI Management:** A suite of intuitive commands (`/system-agent`, `/system-agent:switch`, etc.) for seamless agent operations.
-   **Robust Hooks System:** A library of over 35 hooks providing advanced capabilities like context injection, memory monitoring, and session recovery.
-   **MCP Integration:** Full support for the Model Context Protocol (MCP), enabling agents to utilize tools and communicate effectively.
-   **Terminal-Centric Visual Interface (TUI):** A terminal-based user interface rendered via browser or desktop shell, focused on inspecting and managing agents through a modern, interactive TUI experience, with a full-featured cross-platform desktop application powered by Tauri.
-   **Discovery Engine:** A robust local environment scanner that identifies available AI CLIs, version control, and package managers to provide an objective machine state report.

## Success Metrics
-   **Adoption:** Widespread usage among the Gemini CLI community.
-   **Ecosystem Growth:** A growing library of community-contributed agents and extensions.
-   **Efficiency:** Measurable reduction in user setup time and context switching.
-   **Reliability:** High stability and performance of the core runtime, ensuring a frustration-free experience.
