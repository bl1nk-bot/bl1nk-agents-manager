use crate::agents::types::{AgentName, AgentPromptMetadata, BuiltinAgentName};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailableAgent {
    pub name: AgentName,
    pub description: String,
    pub metadata: AgentPromptMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailableTool {
    pub name: String,
    #[serde(rename = "category")]
    pub category: ToolCategory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolCategory {
    #[serde(rename = "lsp")]
    Lsp,
    #[serde(rename = "ast")]
    Ast,
    #[serde(rename = "search")]
    Search,
    #[serde(rename = "session")]
    Session,
    #[serde(rename = "command")]
    Command,
    #[serde(rename = "other")]
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailableSkill {
    pub name: String,
    pub description: String,
    #[serde(rename = "location")]
    pub location: SkillLocation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SkillLocation {
    #[serde(rename = "user")]
    User,
    #[serde(rename = "project")]
    Project,
    #[serde(rename = "plugin")]
    Plugin,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailableCategory {
    pub name: String,
    pub description: String,
}

pub fn categorize_tools(tool_names: &[String]) -> Vec<AvailableTool> {
    tool_names
        .iter()
        .map(|name| {
            let category = if name.starts_with("lsp_") {
                ToolCategory::Lsp
            } else if name.starts_with("ast_grep") {
                ToolCategory::Ast
            } else if name == "grep" || name == "glob" {
                ToolCategory::Search
            } else if name.starts_with("session_") {
                ToolCategory::Session
            } else if name == "slashcommand" {
                ToolCategory::Command
            } else {
                ToolCategory::Other
            };

            AvailableTool {
                name: name.clone(),
                category,
            }
        })
        .collect()
}

fn format_tools_for_prompt(tools: &[AvailableTool]) -> String {
    let lsp_tools: Vec<&AvailableTool> = tools
        .iter()
        .filter(|t| matches!(t.category, ToolCategory::Lsp))
        .collect();
    let ast_tools: Vec<&AvailableTool> = tools
        .iter()
        .filter(|t| matches!(t.category, ToolCategory::Ast))
        .collect();
    let search_tools: Vec<&AvailableTool> = tools
        .iter()
        .filter(|t| matches!(t.category, ToolCategory::Search))
        .collect();

    let mut parts = Vec::new();

    if !search_tools.is_empty() {
        parts.extend(search_tools.iter().map(|t| format!("`{}`", t.name)));
    }

    if !lsp_tools.is_empty() {
        parts.push("`lsp_*`".to_string());
    }

    if !ast_tools.is_empty() {
        parts.push("`ast_grep`".to_string());
    }

    parts.join(", ")
}

pub fn build_key_triggers_section(agents: &[AvailableAgent], _skills: &[AvailableSkill]) -> String {
    let key_triggers: Vec<String> = agents
        .iter()
        .filter(|a| a.metadata.key_trigger.is_some())
        .map(|a| format!("- {}", a.metadata.key_trigger.as_ref().unwrap()))
        .collect();

    if key_triggers.is_empty() {
        return String::new();
    }

    format!(
        "######### Key Triggers (check BEFORE classification):\n\n{}\n- **\"Look into\" + \"create PR\"** → Not just research. Full implementation cycle expected.",
        key_triggers.join("\n")
    )
}

pub fn build_tool_selection_table(
    agents: &[AvailableAgent],
    tools: &[AvailableTool],
    _skills: &[AvailableSkill],
) -> String {
    let mut rows = vec![
        "######### Tool & Agent Selection:".to_string(),
        "".to_string(),
    ];

    rows.push("| Resource | Cost | When to Use |".to_string());
    rows.push("|----------|------|-------------|".to_string());

    if !tools.is_empty() {
        let tools_display = format_tools_for_prompt(tools);
        rows.push(format!(
            "| {} | FREE | Not Complex, Scope Clear, No Implicit Assumptions |",
            tools_display
        ));
    }

    // Define cost ordering
    let cost_order = |cost: &crate::agents::types::AgentCost| -> i32 {
        match cost {
            crate::agents::types::AgentCost::Free => 0,
            crate::agents::types::AgentCost::Cheap => 1,
            crate::agents::types::AgentCost::Expensive => 2,
        }
    };

    let mut sorted_agents: Vec<&AvailableAgent> = agents
        .iter()
        .filter(|a| a.metadata.category != crate::agents::types::AgentCategory::Utility)
        .collect();

    sorted_agents.sort_by(|a, b| cost_order(&a.metadata.cost).cmp(&cost_order(&b.metadata.cost)));

    for agent in sorted_agents {
        let short_desc = agent
            .description
            .split('.')
            .next()
            .unwrap_or(&agent.description);
        rows.push(format!(
            "| `{}` agent | {} | {} |",
            agent.name,
            match agent.metadata.cost {
                crate::agents::types::AgentCost::Free => "FREE",
                crate::agents::types::AgentCost::Cheap => "CHEAP",
                crate::agents::types::AgentCost::Expensive => "EXPENSIVE",
            },
            short_desc
        ));
    }

    rows.push("".to_string());
    rows.push(
        "**Default flow**: explorer/researcher (background) + tools → expert (if required)"
            .to_string(),
    );

    rows.join("\n")
}

pub fn build_explorer_section(agents: &[AvailableAgent]) -> String {
    let explorer_agent = agents
        .iter()
        .find(|a| matches!(a.name, AgentName::Builtin(BuiltinAgentName::Explorer)));
    if let Some(agent) = explorer_agent {
        let empty_use: Vec<String> = Vec::new();
        let empty_avoid: Vec<String> = Vec::new();
        let use_when = agent.metadata.use_when.as_ref().unwrap_or(&empty_use);
        let avoid_when = agent.metadata.avoid_when.as_ref().unwrap_or(&empty_avoid);

        let mut result = "######### Explorer Agent = Contextual Grep\n\nUse it as a **peer tool**, not a fallback. Fire liberally.\n\n".to_string();
        result.push_str("| Use Direct Tools | Use Explorer Agent |\n");
        result.push_str("|------------------|-------------------|\n");

        for w in avoid_when {
            result.push_str(&format!("| {} |  |\n", w));
        }

        for w in use_when {
            result.push_str(&format!("|  | {} |\n", w));
        }

        return result;
    }

    String::new()
}

pub fn build_researcher_section(agents: &[AvailableAgent]) -> String {
    let researcher_agent = agents
        .iter()
        .find(|a| matches!(a.name, AgentName::Builtin(BuiltinAgentName::Researcher)));
    if let Some(agent) = researcher_agent {
        let empty_use: Vec<String> = Vec::new();
        let use_when = agent.metadata.use_when.as_ref().unwrap_or(&empty_use);

        let mut result = "######### Researcher Agent = Reference Grep\n\nSearch **external references** (docs, OSS, web). Fire proactively when unfamiliar libraries are involved.\n\n".to_string();
        result.push_str("| Contextual Grep (Internal) | Reference Grep (External) |\n");
        result.push_str("|----------------------------|---------------------------|\n");
        result.push_str("| Search OUR codebase | Search EXTERNAL resources |\n");
        result.push_str("| Find patterns in THIS repo | Find examples in OTHER repos |\n");
        result.push_str("| How does our code work? | How does this library work? |\n");
        result.push_str("| Project-specific logic | Official API documentation |\n");
        result.push_str("| | Library best practices & quirks |\n");
        result.push_str("| | OSS implementation examples |\n");
        result.push_str("\n**Trigger phrases** (fire researcher immediately):\n");

        for w in use_when {
            result.push_str(&format!("- \"{}\"\n", w));
        }

        return result;
    }

    String::new()
}

pub fn build_delegation_table(agents: &[AvailableAgent]) -> String {
    let mut rows = vec![
        "######### Delegation Table:".to_string(),
        "".to_string(),
        "| Domain | Delegate To | Trigger |".to_string(),
        "|--------|-------------|---------|".to_string(),
    ];

    for agent in agents {
        for trigger in &agent.metadata.triggers {
            rows.push(format!(
                "| {} | `{}` | {} |",
                trigger.domain, agent.name, trigger.trigger
            ));
        }
    }

    rows.join("\n")
}

pub fn build_category_skills_delegation_guide(
    categories: &[AvailableCategory],
    skills: &[AvailableSkill],
) -> String {
    if categories.is_empty() && skills.is_empty() {
        return String::new();
    }

    let category_rows: Vec<String> = categories
        .iter()
        .map(|c| {
            let desc = if c.description.is_empty() {
                c.name.clone()
            } else {
                c.description.clone()
            };
            format!("| `{}` | {} |", c.name, desc)
        })
        .collect();

    let skill_rows: Vec<String> = skills
        .iter()
        .map(|s| {
            let desc = s.description.split('.').next().unwrap_or(&s.description);
            format!("| `{}` | {} |", s.name, desc)
        })
        .collect();

    format!(
        "######### Category + Skills Delegation System

**delegate_task() combines categories and skills for optimal task execution.**

#### Available Categories (Domain-Optimized Models)

Each category is configured with a model optimized for that domain. Read the description to understand when to use it.

| Category | Domain / Best For |
|----------|-------------------|
{}

#### Available Skills (Domain Expertise Injection)

Skills inject specialized instructions into the subagent. Read the description to understand when each skill applies.

| Skill | Expertise Domain |
|-------|------------------|
{}

---

### MANDATORY: Category + Skill Selection Protocol

**STEP 1: Select Category**
- Read each category's description
- Match task requirements to category domain
- Select the category whose domain BEST fits the task

**STEP 2: Evaluate ALL Skills**
For EVERY skill listed above, ask yourself:
> \"Does this skill's expertise domain overlap with my task?\"

- If YES → INCLUDE in `load_skills=[...]`
- If NO → You MUST justify why (see below)

**STEP 3: Justify Omissions**

If you choose NOT to include a skill that MIGHT be relevant, you MUST provide:

```
SKILL EVALUATION for \"[skill-name]\":
- Skill domain: [what the skill description says]
- Task domain: [what your task is about]
- Decision: OMIT
- Reason: [specific explanation of why domains don't overlap]
```

**WHY JUSTIFICATION IS MANDATORY:**
- Forces you to actually READ skill descriptions
- Prevents lazy omission of potentially useful skills
- Subagents are STATELESS - they only know what you tell them
- Missing a relevant skill = suboptimal output

---

### Delegation Pattern

```typescript
delegate_task(
  category=\"[selected-category]\",
  load_skills=[\"skill-1\", \"skill-2\"],  // Include ALL relevant skills
  prompt=\"...\"
)
```

**ANTI-PATTERN (will produce poor results):**
```typescript
delegate_task(category=\"...\", load_skills=[], prompt=\"...\")  // Empty load_skills without justification
```",
        category_rows.join("\n"),
        skill_rows.join("\n")
    )
}

pub fn build_expert_section(agents: &[AvailableAgent]) -> String {
    let expert_agent = agents
        .iter()
        .find(|a| matches!(a.name, AgentName::Builtin(BuiltinAgentName::Expert)));
    if let Some(agent) = expert_agent {
        let empty_vec = vec![];
        let use_when = agent.metadata.use_when.as_ref().unwrap_or(&empty_vec);
        let avoid_when = agent.metadata.avoid_when.as_ref().unwrap_or(&empty_vec);

        return format!(
            "<Expert_Usage>
## Expert — Read-Only High-IQ Consultant

Expert is a read-only, expensive, high-quality reasoning model for debugging and architecture. Consultation only.

### WHEN to Consult:

| Trigger | Action |
|---------|--------|
{}

### WHEN NOT to Consult:

{}

### Usage Pattern:
Briefly announce \"Consulting Expert for [reason]\" before invocation.

**Exception**: This is the ONLY case where you announce before acting. For all other work, start immediately without status updates.
</Expert_Usage>",
            use_when.iter().map(|w| format!("| {} | Expert FIRST, then implement |", w)).collect::<Vec<_>>().join("\n"),
            avoid_when.iter().map(|w| format!("- {}", w)).collect::<Vec<_>>().join("\n")
        );
    }

    String::new()
}

pub fn build_hard_blocks_section() -> String {
    let blocks = [
        "| Type error suppression (`as any`, `@ts-ignore`) | Never |",
        "| Commit without explicit request | Never |",
        "| Speculate about unread code | Never |",
        "| Leave code in broken state after failures | Never |",
    ];

    format!(
        "####### Hard Blocks (NEVER violate)

| Constraint | No Exceptions |
|------------|---------------|
{}",
        blocks.join("\n")
    )
}

pub fn build_anti_patterns_section() -> String {
    let patterns = [
        "| **Type Safety** | `as any`, `@ts-ignore`, `@ts-expect-error` |",
        "| **Error Handling** | Empty catch blocks `catch(e) {{}}` |",
        "| **Testing** | Deleting failing tests to \"pass\" |",
        "| **Search** | Firing agents for single-line typos or obvious syntax errors |",
        "| **Debugging** | Shotgun debugging, random changes |",
    ];

    format!(
        "####### Anti-Patterns (BLOCKING violations)

| Category | Forbidden |
|----------|-----------|
{}",
        patterns.join("\n")
    )
}

pub fn build_ultrawork_section(
    agents: &[AvailableAgent],
    categories: &[AvailableCategory],
    skills: &[AvailableSkill],
) -> String {
    let mut lines = Vec::new();

    if !categories.is_empty() {
        lines.push("**Categories** (for implementation tasks):".to_string());
        for cat in categories {
            let short_desc = if cat.description.is_empty() {
                cat.name.clone()
            } else {
                cat.description.clone()
            };
            lines.push(format!("- `{}`: {}", cat.name, short_desc));
        }
        lines.push("".to_string());
    }

    if !skills.is_empty() {
        lines
            .push("**Skills** (combine with categories - EVALUATE ALL for relevance):".to_string());
        for skill in skills {
            let short_desc = skill
                .description
                .split('.')
                .next()
                .unwrap_or(&skill.description);
            lines.push(format!("- `{}`: {}", skill.name, short_desc));
        }
        lines.push("".to_string());
    }

    if !agents.is_empty() {
        // Sort agents with priority for specific types
        let ultrawork_agent_priority = ["explorer", "researcher", "planner", "expert"];
        let mut sorted_agents = agents.to_vec();
        sorted_agents.sort_by(|a, b| {
            let a_name = a.name.to_string();
            let b_name = b.name.to_string();

            let a_idx = ultrawork_agent_priority
                .iter()
                .position(|&x| x == a_name.as_str());
            let b_idx = ultrawork_agent_priority
                .iter()
                .position(|&x| x == b_name.as_str());

            match (a_idx, b_idx) {
                (Some(a_pos), Some(b_pos)) => a_pos.cmp(&b_pos),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            }
        });

        lines.push("**Agents** (for specialized consultation/exploration):".to_string());
        for agent in sorted_agents {
            let short_desc = agent
                .description
                .split('.')
                .next()
                .unwrap_or(&agent.description);
            let suffix =
                if agent.name.to_string() == "explorer" || agent.name.to_string() == "researcher" {
                    " (multiple)"
                } else {
                    ""
                };
            lines.push(format!("- `{}{}`: {}", agent.name, suffix, short_desc));
        }
    }

    lines.join("\n")
}

// Helper for sorting
use std::cmp::Ordering;
impl std::cmp::PartialEq for AvailableAgent {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl std::cmp::Eq for AvailableAgent {}

impl std::cmp::PartialOrd for AvailableAgent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::Ord for AvailableAgent {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.to_string().cmp(&other.name.to_string())
    }
}
