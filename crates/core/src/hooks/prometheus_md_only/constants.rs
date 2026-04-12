use crate::hooks::shared::system_directive::{create_system_directive, SystemDirectiveType};
use crate::hooks::shared::agent_display_names::get_agent_display_name;

pub const HOOK_NAME: &str = "prometheus-md-only";

pub const PROMETHEUS_AGENTS: &[&str] = &["prometheus"];

pub const ALLOWED_EXTENSIONS: &[&str] = &[".md"];

pub const ALLOWED_PATH_PREFIX: &str = ".sisyphus";

pub const BLOCKED_TOOLS: &[&str] = &["Write", "Edit", "write", "edit"];

pub const PLANNING_CONSULT_WARNING: &str = "

---

[SYSTEM_DIRECTIVE:prometheus_read_only]

You are being invoked by Prometheus (Prometheus), a READ-ONLY planning agent.

**CRITICAL CONSTRAINTS:**
- DO NOT modify any files (no Write, Edit, or any file mutations)
- DO NOT execute commands that change system state
- DO NOT create, delete, or rename files
- ONLY provide analysis, recommendations, and information

**YOUR ROLE**: Provide consultation, research, and analysis to assist with planning.
Return your findings and recommendations. The actual implementation will be handled separately after planning is complete.

---

";

pub const PROMETHEUS_WORKFLOW_REMINDER: &str = "

---

[SYSTEM_DIRECTIVE:prometheus_read_only]

## PROMETHEUS MANDATORY WORKFLOW REMINDER

**You are writing a work plan. STOP AND VERIFY you completed ALL steps:**

┌─────────────────────────────────────────────────────────────────────┐
│                     PROMETHEUS WORKFLOW                             │
├──────┬──────────────────────────────────────────────────────────────┤
│  1   │ INTERVIEW: Full consultation with user                       │
│      │    - Gather ALL requirements                                 │
│      │    - Clarify ambiguities                                     │
│      │    - Record decisions to .sisyphus/drafts/                   │
├──────┼──────────────────────────────────────────────────────────────┤
│  2   │ METIS CONSULTATION: Pre-generation gap analysis              │
│      │    - delegate_task(agent=\"Metis (Plan Consultant)\", ...)     │
│      │    - Identify missed questions, guardrails, assumptions      │
├──────┼──────────────────────────────────────────────────────────────┤
│  3   │ PLAN GENERATION: Write to .sisyphus/plans/*.md               │
│      │    <- YOU ARE HERE                                           │
├──────┼──────────────────────────────────────────────────────────────┤
│  4   │ MOMUS REVIEW (if high accuracy requested)                    │
│      │    - delegate_task(agent=\"Momus (Plan Reviewer)\", ...)       │
│      │    - Loop until OKAY verdict                                 │
├──────┼──────────────────────────────────────────────────────────────┤
│  5   │ SUMMARY: Present to user                                     │
│      │    - Key decisions made                                      │
│      │    - Scope IN/OUT                                            │
│      │    - Offer: \"Start Work\" vs \"High Accuracy Review\"           │
│      │    - Guide to /start-work                                    │
└──────┴──────────────────────────────────────────────────────────────┘

**DID YOU COMPLETE STEPS 1-2 BEFORE WRITING THIS PLAN?**
**AFTER WRITING, WILL YOU DO STEPS 4-5?**

If you skipped steps, STOP NOW. Go back and complete them.

---

";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(HOOK_NAME, "prometheus-md-only");
        assert!(PROMETHEUS_AGENTS.contains(&"prometheus"));
        assert!(ALLOWED_EXTENSIONS.contains(&".md"));
        assert_eq!(ALLOWED_PATH_PREFIX, ".sisyphus");
        assert!(BLOCKED_TOOLS.contains(&"Write"));
        assert!(BLOCKED_TOOLS.contains(&"Edit"));
        assert!(PLANNING_CONSULT_WARNING.contains("READ-ONLY planning agent"));
        assert!(PROMETHEUS_WORKFLOW_REMINDER.contains("PROMETHEUS WORKFLOW"));
    }
}