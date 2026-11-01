# Specification: Resource-Aware Router & Planner (Empowered Selection)

## 1. Overview
The final level of the "Brain" evolution. This track enables the system to generate implementation plans (Proposals) based on the "Truth" of local resources and agent costs. Instead of executing blindly, the system presents its strategy to the user and awaits confirmation, ensuring the user remains in control.

## 2. Functional Requirements
- **Dynamic Planning (Proposals):**
    - When a task is received, the Router analyzes the `AgentRegistry` (Active state).
    - Generate a `ResourcePlan` suggesting specific agents based on:
        - Tool Readiness (Ready agents preferred).
        - Cost (Cheaper agents preferred for sub-tasks).
        - Expertise (Capability match).
- **Interactive Approval:**
    - Provide a new UI state/command to display the `ResourcePlan`.
    - Wait for user `Yes/No` or "Modify" before proceeding to execution.
- **Cost Awareness:**
    - Show estimated cost category (FREE/CHEAP/EXPENSIVE) for the proposed plan.

## 3. Non-Functional Requirements
- **User Agency:** The system *suggests*, it does not *enforce*. If a user wants to run a "Missing Tool" agent, let them (and let the session handle the inevitable error).
- **Explainability:** The proposal must include a "Reasoning" string explaining *why* these agents were chosen.

## 4. Acceptance Criteria
- Router can generate a structured `PlanProposal`.
- TUI can display this proposal in a readable format.
- Execution only begins after user confirmation.
