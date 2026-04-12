use crate::agents::types::{AgentCategory, AgentConfig, AgentCost, AgentPromptMetadata};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref OBSERVER_PROMPT_METADATA: AgentPromptMetadata = AgentPromptMetadata {
        category: AgentCategory::Utility,
        cost: AgentCost::Cheap,
        prompt_alias: Some("Observer".to_string()),
        key_trigger: None,
        triggers: vec![],
        use_when: None,
        avoid_when: None,
        dedicated_section: None,
    };
}

pub fn create_observer_agent(model: &str) -> AgentConfig {
    let restrictions = create_agent_tool_allowlist(&["read"]);

    AgentConfig {
        description: Some("Analyze media files (PDFs, images, diagrams) that require interpretation beyond raw text. Extracts specific information or summaries from documents, describes visual content. Use when you need analyzed/extracted data rather than literal file contents.".to_string()),
        mode: Some("subagent".to_string()),
        model: Some(model.to_string()),
        temperature: Some(0.1),
        permission: restrictions.permission,
        prompt: Some(get_observer_prompt()),
        id: "observer".to_string(),
        name: "Observer".to_string(),
        agent_type: "omo".to_string(),
        command: None,
        args: None,
        extension_name: None,
        rate_limit: crate::agents::types::RateLimit::default(),
        capabilities: vec!["media-analysis".to_string(), "document-processing".to_string(), "image-description".to_string()],
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

fn create_agent_tool_allowlist(allowed_tools: &[&str]) -> AgentConfig {
    let mut permission = std::collections::HashMap::new();

    // Allow only the specified tools, deny others by default
    for tool in allowed_tools {
        permission.insert(tool.to_string(), "allow".to_string());
    }

    AgentConfig {
        permission: Some(permission),
        ..Default::default()
    }
}

fn get_observer_prompt() -> String {
    r####"You interpret media files that cannot be read as plain text.

Your job: examine the attached file and extract ONLY what was requested.

When to use you:
- Media files the Read tool cannot interpret
- Extracting specific information or summaries from documents
- Describing visual content in images or diagrams
- When analyzed/extracted data is needed, not raw file contents

When NOT to use you:
- Source code or plain text files needing exact contents (use Read)
- Files that need editing afterward (need literal content from Read)
- Simple file reading where no interpretation is needed

How you work:
1. Receive a file path and a goal describing what to extract
2. Read and analyze the file deeply
3. Return ONLY the relevant extracted information
4. The main agent never processes the raw file - you save context tokens

For PDFs: extract text, structure, tables, data from specific sections
For images: describe layouts, UI elements, text, diagrams, charts
For diagrams: explain relationships, flows, architecture depicted

Response rules:
- Return extracted information directly, no preamble
- If info not found, state clearly what's missing
- Match the language of the request
- Be thorough on the goal, concise on everything else

Your output goes straight to the main agent for continued work."####
        .to_string()
}
