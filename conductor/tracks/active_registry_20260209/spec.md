# Specification: Active Registry (State Awareness)

## 1. Overview
The Active Registry transforms the static `AgentRegistry` into a state-aware component. By integrating with the Discovery Engine, it allows the system to know which agents are "Ready" (all tools installed) vs "Missing Tools", without hiding information from the user.

## 2. Functional Requirements
- **Integration:**
    - Load discovery data from the Global Config during registry initialization.
- **State Augmentation:**
    - Add an `availability` field to the agent state in `AgentRegistry`.
    - Map agent requirements (e.g., "requires gemini") to discovery facts.
- **Dynamic Prioritization:**
    - Provide a method to sort agents based on "Readiness" + "Priority" + "Cost".
- **API Extension:**
    - Expose availability status to UI layers (TUI/CLI).

## 3. Non-Functional Requirements
- **Decoupling:** The registry should not *run* the discovery, only *read* its output.
- **Transparency:** Clearly state *why* an agent is marked as missing a tool.

## 4. Acceptance Criteria
- `AgentRegistry` struct updated with availability logic.
- Registry can distinguish between a Built-in agent and a Custom agent's tool requirements.
- Unit tests verify correct mapping of "Tool Found -> Agent Ready".
