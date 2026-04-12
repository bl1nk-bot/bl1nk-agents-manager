use crate::agents::types::{
    AgentCategory, AgentConfig, AgentCost, AgentPromptMetadata, DelegationTrigger,
};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref EXPLORER_PROMPT_METADATA: AgentPromptMetadata = AgentPromptMetadata {
        category: AgentCategory::Exploration,
        cost: AgentCost::Free,
        prompt_alias: Some("Explorer".to_string()),
        key_trigger: Some("2+ modules involved → fire `explorer` background".to_string()),
        triggers: vec![DelegationTrigger {
            domain: "Explorer".to_string(),
            trigger: "Find existing codebase structure, patterns and styles".to_string(),
        }],
        use_when: Some(vec![
            "Multiple search angles needed".to_string(),
            "Unfamiliar module structure".to_string(),
            "Cross-layer pattern discovery".to_string(),
        ]),
        avoid_when: Some(vec![
            "You know exactly what to search".to_string(),
            "Single keyword/pattern suffices".to_string(),
            "Known file location".to_string(),
        ]),
        dedicated_section: None,
    };
}

pub fn create_explorer_agent(model: &str) -> AgentConfig {
    // In Rust, we'll create a simplified version of tool restrictions
    // This would need to be expanded based on the actual implementation of createAgentToolRestrictions
    let restrictions = create_agent_tool_restrictions(&[
        "write",
        "edit",
        "task",
        "delegate_task",
        "call_ome_agent",
    ]);

    AgentConfig {
        description: Some("Contextual grep for codebases. Answers \"Where is X?\", \"Which file has Y?\", \"Find the code that does Z\". Fire multiple in parallel for broad searches. Specify thoroughness: \"quick\" for basic, \"medium\" for moderate, \"very thorough\" for comprehensive analysis.".to_string()),
        mode: Some("subagent".to_string()),
        model: Some(model.to_string()),
        temperature: Some(0.1),
        permission: restrictions.permission,
        prompt: Some(get_explorer_prompt()),
        id: "explorer".to_string(),
        name: "Explorer".to_string(),
        agent_type: "omo".to_string(),
        command: None,
        args: None,
        extension_name: None,
        rate_limit: crate::agents::types::RateLimit::default(),
        capabilities: vec!["search".to_string(), "codebase-analysis".to_string()],
        priority: 50,
        enabled: true,
        max_tokens: None,
        color: None,
        thinking: None,
        reasoning_effort: None,
        skills: None,
        text_verbosity: None,
    }
}

fn create_agent_tool_restrictions(restricted_tools: &[&str]) -> AgentConfig {
    // This is a simplified implementation - in a real system, this would be more complex
    let mut permission = std::collections::HashMap::new();

    // Deny the restricted tools
    for tool in restricted_tools {
        permission.insert(tool.to_string(), "deny".to_string());
    }

    AgentConfig {
        permission: Some(permission),
        ..Default::default()
    }
}

fn get_explorer_prompt() -> String {
    r####"You are a codebase search specialist. Your job: find files and code, return actionable results.

## Your Mission

Answer questions like:
- "Where is X implemented?"
- "Which files contain Y?"
- "Find the code that does Z"

## CRITICAL: What You Must Deliver

Every response MUST include:

### 1. Intent Analysis (Required)
Before ANY search, wrap your analysis in <analysis> tags:

<analysis>
**Literal Request**: [What they literally asked]
**Actual Need**: [What they're really trying to accomplish]
**Success Looks Like**: [What result would let them proceed immediately]
</analysis>

### 2. Parallel Execution (Required)
Launch **3+ tools simultaneously** in your first action. Never sequential unless output depends on prior result.

### 3. Structured Results (Required)
Always end with this exact format:

<results>
<files>
- /absolute/path/to/file1.ts — [why this file is relevant]
- /absolute/path/to/file2.ts — [why this file is relevant]
</files>

<answer>
[Direct answer to their actual need, not just file list]
[If they asked "where is auth?", explain the auth flow you found]
</answer>

<next_steps>
[What they should do with this information]
[Or: "Ready to proceed - no follow-up needed"]
</next_steps>
</results>

## Success Criteria

| Criterion | Requirement |
|-----------|-------------|
| **Paths** | ALL paths must be **absolute** (start with /) |
| **Completeness** | Find ALL relevant matches, not just the first one |
| **Actionability** | Caller can proceed **without asking follow-up questions** |
| **Intent** | Address their **actual need**, not just literal request |

## Failure Conditions

Your response has **FAILED** if:
- Any path is relative (not absolute)
- You missed obvious matches in the codebase
- Caller needs to ask "but where is exactly?" or "what about X?"
- You only answered the literal question, not the underlying need
- No <results> block with structured output

## Constraints

- **Read-only**: You cannot create, modify, or delete files
- **No emojis**: Keep output clean and parseable
- **No file creation**: Report findings as message text, never write files

## Tool Strategy

Use the right tool for the job:
- **Semantic search** (definitions, references): LSP tools
- **Structural patterns** (function shapes, class structures): ast_grep_search
- **Text patterns** (strings, comments, logs): grep
- **File patterns** (find by name/extension): glob
- **History/evolution** (when added, who changed): git commands

Flood with parallel calls. Cross-validate findings across multiple tools."####.to_string()
}
