pub mod todo_continuation_enforcer;
pub mod context_window_monitor;
pub mod session_notification;
pub mod session_recovery;
pub mod comment_checker;
pub mod tool_output_truncator;
pub mod directory_agents_injector;
pub mod directory_readme_injector;
pub mod empty_task_response_detector;
pub mod anthropic_context_window_limit_recovery;

pub mod compaction_context_injector;
pub mod think_mode;
pub mod claude_code_hooks;
pub mod rules_injector;
pub mod background_notification;
pub mod auto_update_checker;

pub mod agent_usage_reminder;
pub mod keyword_detector;
pub mod non_interactive_env;
pub mod interactive_bash_session;

pub mod thinking_block_validator;
pub mod category_skill_reminder;
pub mod ralph_loop;
pub mod auto_slash_command;
pub mod edit_error_recovery;
pub mod prometheus_md_only;
pub mod sisyphus_junior_notepad;
pub mod task_resume_info;
pub mod start_work;
pub mod atlas;
pub mod delegate_task_retry;
pub mod question_label_truncator;
pub mod shared;
pub mod hook_message_injector;
pub mod claude_code_session_state;

// Re-exports for convenience
pub use todo_continuation_enforcer::TodoContinuationEnforcerHook;
pub use context_window_monitor::ContextWindowMonitorHook;
pub use session_notification::SessionNotificationHook;
pub use session_recovery::SessionRecoveryHook;
pub use comment_checker::CommentCheckerHook;
pub use tool_output_truncator::ToolOutputTruncatorHook;
pub use directory_agents_injector::DirectoryAgentsInjectorHook;
pub use directory_readme_injector::DirectoryReadmeInjectorHook;
pub use empty_task_response_detector::EmptyTaskResponseDetectorHook;
pub use anthropic_context_window_limit_recovery::AnthropicContextWindowLimitRecoveryHook;

pub use compaction_context_injector::CompactionContextInjectorHook;
pub use think_mode::ThinkModeHook;
pub use claude_code_hooks::ClaudeCodeHooksHook;
pub use rules_injector::RulesInjectorHook;
pub use background_notification::BackgroundNotificationHook;
pub use auto_update_checker::AutoUpdateCheckerHook;

pub use agent_usage_reminder::AgentUsageReminderHook;
pub use keyword_detector::KeywordDetectorHook;
pub use non_interactive_env::NonInteractiveEnvHook;
pub use interactive_bash_session::InteractiveBashSessionHook;

pub use thinking_block_validator::ThinkingBlockValidatorHook;
pub use category_skill_reminder::CategorySkillReminderHook;
pub use ralph_loop::RalphLoopHook;
pub use auto_slash_command::AutoSlashCommandHook;
pub use edit_error_recovery::EditErrorRecoveryHook;
pub use prometheus_md_only::PrometheusMdOnlyHook;
pub use sisyphus_junior_notepad::SisyphusJuniorNotepadHook;
pub use task_resume_info::TaskResumeInfoHook;
pub use start_work::StartWorkHook;
pub use atlas::AtlasHook;
pub use delegate_task_retry::DelegateTaskRetryHook;
pub use question_label_truncator::QuestionLabelTruncatorHook;

#[cfg(test)]
mod tests {
    #[test]
    fn test_hooks_lib() {
        // Basic test to ensure the module compiles
        assert!(true);
    }
}