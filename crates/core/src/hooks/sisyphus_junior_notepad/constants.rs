pub const HOOK_NAME: &str = "sisyphus-junior-notepad";

pub const NOTEPAD_DIRECTIVE: &str = "
<Work_Context>
## Notepad Location (for recording learnings)
NOTEPAD PATH: .sisyphus/notepads/{plan-name}/
- learnings.md: Record patterns, conventions, successful approaches
- issues.md: Record problems, blockers, gotchas encountered
- decisions.md: Record architectural choices and rationales
- problems.md: Record unresolved issues, technical debt

You SHOULD append findings to notepad files after completing work.
IMPORTANT: Always APPEND to notepad files - never overwrite or use Edit tool.

## Plan Location (READ ONLY)
PLAN PATH: .sisyphus/plans/{plan-name}.md

CRITICAL RULE: NEVER MODIFY THE PLAN FILE

The plan file (.sisyphus/plans/*.md) is SACRED and READ-ONLY.
- You may READ the plan to understand tasks
- You may READ checkbox items to know what to do
- You MUST NOT edit, modify, or update the plan file
- You MUST NOT mark checkboxes as complete in the plan
- Only the Orchestrator manages the plan file

VIOLATION = IMMEDIATE FAILURE. The Orchestrator tracks plan state.
</Work_Context>
";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(HOOK_NAME, "sisyphus-junior-notepad");
        assert!(NOTEPAD_DIRECTIVE.contains("Notepad Location"));
        assert!(NOTEPAD_DIRECTIVE.contains(".sisyphus/notepads"));
        assert!(NOTEPAD_DIRECTIVE.contains("learnings.md"));
        assert!(NOTEPAD_DIRECTIVE.contains("issues.md"));
        assert!(NOTEPAD_DIRECTIVE.contains("decisions.md"));
        assert!(NOTEPAD_DIRECTIVE.contains("problems.md"));
        assert!(NOTEPAD_DIRECTIVE.contains("Plan Location"));
        assert!(NOTEPAD_DIRECTIVE.contains("NEVER MODIFY THE PLAN FILE"));
    }
}