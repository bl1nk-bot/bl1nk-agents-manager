This document defines the clean architecture for the Multi-Bot Management System on Poe.

## 1. Core Components

| Component | Responsibility | Key Features |
| :--- | :--- | :--- |
| **Bot Registry** | Centralized storage for bot metadata and state. | CRUD operations, session tracking, skill inventory. |
| **Bot Creation Factory** | Logic for instantiating different bot types. | Template-based creation, dynamic skill injection. |
| **Execution Engine** | Orchestrates bot runs and resource allocation. | Parallel/Sequential execution, Load balancing. |
| **Prompt & Script Manager** | Manages prompt templates and script generation. | Real-time editing, versioning, parameter validation. |

## 2. Bot Types

- **Inline Bots**: Single-purpose, quick execution (e.g., Translator, Formatter).
- **Skill Bots**: Multi-capability bots composed of multiple skills.
- **Creative Bots**: Specialized for content generation and ideation.

## 3. Execution Strategies

- **Parallel**: Run multiple bots simultaneously and merge results.
- **Sequential (Pipeline)**: Chain bots where output of one is input to next.
- **Router**: Intelligent intent detection to select the best bot.
