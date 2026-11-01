## OVERVIEW

10 AI agents for multi-model orchestration. Orchestrator (primary), Manager (orchestrator), Expert, Researcher, Explorer, Observer, Planner, Consultant, Auditor, Orchestrator-Junior.

## STRUCTURE

```
agents/
├── manager.rs                  # Master Orchestrator (holds todo list)
├── orchestrator.rs             # Main prompt (SF Bay Area engineer identity)
├── orchestrator_junior.rs      # Delegated task executor (category-spawned)
├── expert.rs                   # Strategic advisor
├── researcher.rs               # Multi-repo research
├── explorer.rs                 # Fast contextual grep
├── observer.rs                 # Media analyzer
├── planner.rs                  # Strategic planning (Interview/Consultant mode)
├── consultant.rs               # Pre-planning analysis (Gap detection)
├── auditor.rs                  # Plan reviewer (Ruthless fault-finding)
├── prompt_builder.rs               # Dynamic prompt generation
├── types.rs                    # AgentModelConfig, AgentPromptMetadata
├── utils.rs                    # create_builtin_agents(), build_agent()
└── mod.rs                      # builtin_agents export
```

## AGENT MODELS

| Agent | Model | Temp | Purpose |
| :--- | :--- | :--- | :--- |
| **Orchestrator** | anthropic/claude-opus-4-5 | 0.1 | Primary orchestrator |
| **Manager** | anthropic/claude-opus-4-5 | 0.1 | Master orchestrator |
| **Expert** | openai/gpt-5.2 | 0.1 | Consultation, debugging |
| **Researcher** | opencode/big-pickle | 0.1 | Docs, GitHub search |
| **Explorer** | opencode/gpt-5-nano | 0.1 | Fast contextual grep |
| **Observer** | google/gemini-3-flash | 0.1 | PDF/image analysis |
| **Planner** | anthropic/claude-opus-4-5 | 0.1 | Strategic planning |
| **Consultant** | anthropic/claude-sonnet-4-5 | 0.3 | Pre-planning analysis |
| **Auditor** | anthropic/claude-sonnet-4-5 | 0.1 | Plan validation |
| **Orchestrator-Junior** | anthropic/claude-sonnet-4-5 | 0.1 | Category-spawned executor |

## PATTERNS

- **Factory**: `create_XXX_agent(model: &str): AgentConfig`
- **Metadata**: `XXX_PROMPT_METADATA` with category, cost, triggers.
- **Thinking**: 32k budget tokens for Orchestrator, Expert, Planner, Manager.

## ANTI-PATTERNS

- **Trust reports**: NEVER trust "I'm done" - verify outputs.
- **High temp**: Don't use >0.3 for code agents.
- **Sequential calls**: Use `delegate_task` with `run_in_background` for exploration.
- **Planner writing code**: Planner only - never implements.
