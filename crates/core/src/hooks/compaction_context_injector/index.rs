use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::hooks::hook_message_injector::{inject_hook_message, HookMessageOptions, ModelInfo, PathInfo};
use crate::hooks::shared::system_directive::{create_system_directive, SystemDirectiveType};
use crate::hooks::shared::logger::log;

const HOOK_NAME: &str = "compaction-context-injector";

const SUMMARIZE_CONTEXT_PROMPT: &str = "[SYSTEM_DIRECTIVE:compaction_context]

When summarizing this session, you MUST include the following sections in your summary:

## 1. User Requests (As-Is)
- List all original user requests exactly as they were stated
- Preserve the user's exact wording and intent

## 2. Final Goal
- What the user ultimately wanted to achieve
- The end result or deliverable expected

## 3. Work Completed
- What has been done so far
- Files created/modified
- Features implemented
- Problems solved

## 4. Remaining Tasks
- What still needs to be done
- Pending items from the original request
- Follow-up tasks identified during the work

## 5. Active Working Context (For Seamless Continuation)
- **Files**: Paths of files currently being edited or frequently referenced
- **Code in Progress**: Key code snippets, function signatures, or data structures under active development
- **External References**: Documentation URLs, library APIs, or external resources being consulted
- **State & Variables**: Important variable names, configuration values, or runtime state relevant to ongoing work

## 6. MUST NOT Do (Critical Constraints)
- Things that were explicitly forbidden
- Approaches that failed and should not be retried
- User's explicit restrictions or preferences
- Anti-patterns identified during the session

This context is critical for maintaining continuity after compaction.
";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummarizeContext {
    pub session_id: String,
    pub provider_id: String,
    pub model_id: String,
    pub usage_ratio: f64,
    pub directory: String,
}

pub struct CompactionContextInjectorHook;

impl CompactionContextInjectorHook {
    pub fn new() -> Self {
        Self
    }

    pub async fn execute_compaction_context_injection(
        &self,
        ctx: &SummarizeContext,
    ) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("{} injecting context. sessionID={}", HOOK_NAME, ctx.session_id);

        let success = inject_hook_message(
            &ctx.session_id,
            SUMMARIZE_CONTEXT_PROMPT,
            &HookMessageOptions {
                agent: Some("general".to_string()),
                model: Some(ModelInfo {
                    provider_id: ctx.provider_id.clone(),
                    model_id: ctx.model_id.clone(),
                }),
                path: Some(PathInfo {
                    cwd: ctx.directory.clone(),
                }),
            },
        ).await;

        if success {
            log::info!("{} context injected. sessionID={}", HOOK_NAME, ctx.session_id);
        } else {
            log::warn!("{} injection failed. sessionID={}", HOOK_NAME, ctx.session_id);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_execute_compaction_context_injection() {
        let hook = CompactionContextInjectorHook::new();
        let context = SummarizeContext {
            session_id: "test_session".to_string(),
            provider_id: "test_provider".to_string(),
            model_id: "test_model".to_string(),
            usage_ratio: 0.8,
            directory: "/tmp/test".to_string(),
        };

        let result = hook.execute_compaction_context_injection(&context).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_constant_values() {
        assert_eq!(HOOK_NAME, "compaction-context-injector");
        assert!(SUMMARIZE_CONTEXT_PROMPT.contains("User Requests"));
        assert!(SUMMARIZE_CONTEXT_PROMPT.contains("Final Goal"));
        assert!(SUMMARIZE_CONTEXT_PROMPT.contains("Work Completed"));
        assert!(SUMMARIZE_CONTEXT_PROMPT.contains("Remaining Tasks"));
        assert!(SUMMARIZE_CONTEXT_PROMPT.contains("Active Working Context"));
        assert!(SUMMARIZE_CONTEXT_PROMPT.contains("MUST NOT Do"));
    }
}