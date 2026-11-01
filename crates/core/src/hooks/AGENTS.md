## Overview

The hooks system provides specialized functionality for managing various aspects of AI agent operations, including:
- Context window management and recovery
- Session state management
- Tool execution handling
- Error recovery mechanisms
- Workflow orchestration

## Available Agents

Each subdirectory contains a specialized agent:

- `anthropic_context_window_limit_recovery`: Manages recovery from Anthropic context window limit errors
- `claude_code_hooks`: Handles Claude-specific code operations and hooks
- `interactive_bash_session`: Manages interactive bash session operations
- `start_work`: Handles work session initialization
- `task_resume_info`: Manages task resumption information
- `delegate_task_retry`: Handles delegate task failure recovery
- `edit_error_recovery`: Manages recovery from edit tool errors
- `category_skill_reminder`: Provides category and skill usage reminders
- `prometheus_md_only`: Enforces Markdown-only operations for Prometheus agent
- `compaction_context_injector`: Injects context for session compaction
- `sisyphus_junior_notepad`: Manages notepad functionality for Sisyphus Junior
- `question_label_truncator`: Handles question label truncation
- `ralph_loop`: Implements the Ralph Loop pattern
- `session_recovery`: Manages session recovery operations
- `keyword_detector`: Detects keywords in user input
- `rules_injector`: Injects rules and constraints into sessions
- `agent_usage_reminder`: Provides agent usage reminders
- `comment_checker`: Validates code comments
- `directory_agents_injector`: Injects agents based on directory context
- `directory_readme_injector`: Injects README content based on directory context
- `auto_slash_command`: Handles automatic slash command processing
- `auto_update_checker`: Manages automatic update checking
- `background_notification`: Handles background task notifications
- `non_interactive_env`: Manages operations in non-interactive environments
- `atlas`: Orchestrates complex multi-agent workflows
- `think_mode`: Manages thinking block operations
- `thinking_block_validator`: Validates thinking block structure

## Usage

Each agent is automatically invoked when its specific conditions are met. The system intelligently routes operations to the appropriate agents based on context and requirements.

## Architecture & Lifecycle

### Hook Events
| Event | Timing | Can Block | Use Case |
|-------|--------|-----------|----------|
| UserPromptSubmit | `chat.message` | Yes | Keyword detection, slash commands |
| PreToolUse | `tool.execute.before` | Yes | Validate/modify inputs, inject context |
| PostToolUse | `tool.execute.after` | No | Truncate output, error recovery |
| Stop | `event` (session.stop) | No | Auto-continue, notifications |
| onSummarize | Compaction | No | Preserve state, inject summary context |

### Execution Flow
- **UserPromptSubmit**: keywordDetector → claudeCodeHooks → autoSlashCommand → startWork
- **PreToolUse**: questionLabelTruncator → claudeCodeHooks → nonInteractiveEnv → commentChecker → directoryAgentsInjector → directoryReadmeInjector → rulesInjector → prometheusMdOnly → sisyphusJuniorNotepad → atlasHook
- **PostToolUse**: claudeCodeHooks → toolOutputTruncator → contextWindowMonitor → commentChecker → directoryAgentsInjector → directoryReadmeInjector → rulesInjector → emptyTaskResponseDetector → agentUsageReminder → interactiveBashSession → editErrorRecovery → delegateTaskRetry → atlasHook → taskResumeInfo

## How to Add a New Hook
1. Create a new directory in `crates/core/src/hooks/<hook_name_snake_case>/`
2. Create `mod.rs` and implementation files (e.g., `index.rs`)
3. Register in `crates/core/src/hooks/mod.rs`
4. Add to `lib.rs` exports if necessary

## Implementation Patterns

**Simple Single-Event (Rust)**:
```rust
pub struct MyHook;
impl MyHook {
    pub fn on_tool_execute_after(&self, ...) { ... }
}
```

**Stateful Hook**:
```rust
pub struct MyStatefulHook {
    state: Arc<RwLock<HashMap<String, State>>>,
}
```
