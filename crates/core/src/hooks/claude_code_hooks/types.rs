use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ClaudeHookEvent {
    PreToolUse,
    PostToolUse,
    UserPromptSubmit,
    Stop,
    PreCompact,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HookMatcher {
    pub matcher: String,
    pub hooks: Vec<HookCommand>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HookCommand {
    #[serde(rename = "type")]
    pub command_type: String,
    pub command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeHooksConfig {
    #[serde(rename = "PreToolUse")]
    pub pre_tool_use: Option<Vec<HookMatcher>>,
    
    #[serde(rename = "PostToolUse")]
    pub post_tool_use: Option<Vec<HookMatcher>>,
    
    #[serde(rename = "UserPromptSubmit")]
    pub user_prompt_submit: Option<Vec<HookMatcher>>,
    
    #[serde(rename = "Stop")]
    pub stop: Option<Vec<HookMatcher>>,
    
    #[serde(rename = "PreCompact")]
    pub pre_compact: Option<Vec<HookMatcher>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreToolUseInput {
    pub session_id: String,
    pub transcript_path: Option<String>,
    pub cwd: String,
    pub permission_mode: Option<PermissionMode>,
    pub hook_event_name: String, // "PreToolUse"
    pub tool_name: String,
    pub tool_input: HashMap<String, serde_json::Value>,
    pub tool_use_id: Option<String>,
    pub hook_source: Option<HookSource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostToolUseInput {
    pub session_id: String,
    pub transcript_path: Option<String>,
    pub cwd: String,
    pub permission_mode: Option<PermissionMode>,
    pub hook_event_name: String, // "PostToolUse"
    pub tool_name: String,
    pub tool_input: HashMap<String, serde_json::Value>,
    pub tool_response: ToolResponse,
    pub tool_use_id: Option<String>,
    pub hook_source: Option<HookSource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolResponse {
    pub title: Option<String>,
    pub output: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserPromptSubmitInput {
    pub session_id: String,
    pub cwd: String,
    pub permission_mode: Option<PermissionMode>,
    pub hook_event_name: String, // "UserPromptSubmit"
    pub prompt: String,
    pub session: Option<SessionInfo>,
    pub hook_source: Option<HookSource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionInfo {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PermissionMode {
    #[serde(rename = "default")]
    Default,
    #[serde(rename = "plan")]
    Plan,
    #[serde(rename = "acceptEdits")]
    AcceptEdits,
    #[serde(rename = "bypassPermissions")]
    BypassPermissions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HookSource {
    #[serde(rename = "opencode-plugin")]
    OpencodePlugin,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StopInput {
    pub session_id: String,
    pub transcript_path: Option<String>,
    pub cwd: String,
    pub permission_mode: Option<PermissionMode>,
    pub hook_event_name: String, // "Stop"
    pub stop_hook_active: bool,
    pub todo_path: Option<String>,
    pub hook_source: Option<HookSource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreCompactInput {
    pub session_id: String,
    pub cwd: String,
    pub hook_event_name: String, // "PreCompact"
    pub hook_source: Option<HookSource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PermissionDecision {
    #[serde(rename = "allow")]
    Allow,
    #[serde(rename = "deny")]
    Deny,
    #[serde(rename = "ask")]
    Ask,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HookCommonOutput {
    /// If false, Claude stops entirely
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continue_execution: Option<bool>,
    
    /// Message shown to user when continue=false
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,
    
    /// Suppress output from transcript
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suppress_output: Option<bool>,
    
    /// Warning/message displayed to user
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreToolUseOutput {
    #[serde(flatten)]
    pub common: HookCommonOutput,
    
    /// Deprecated: use hook_specific_output.permission_decision instead
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision: Option<String>,
    
    /// Deprecated: use hook_specific_output.permission_decision_reason instead
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hook_specific_output: Option<PreToolUseHookSpecificOutput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreToolUseHookSpecificOutput {
    pub hook_event_name: String, // "PreToolUse"
    pub permission_decision: PermissionDecision,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_decision_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_input: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostToolUseOutput {
    #[serde(flatten)]
    pub common: HookCommonOutput,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hook_specific_output: Option<PostToolUseHookSpecificOutput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostToolUseHookSpecificOutput {
    pub hook_event_name: String, // "PostToolUse"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_context: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookResult {
    pub exit_code: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stdout: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stderr: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptEntry {
    #[serde(rename = "type")]
    pub entry_type: String, // "tool_use", "tool_result", "user", "assistant"
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_input: Option<HashMap<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_output: Option<HashMap<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TodoItem {
    pub id: String,
    pub content: String,
    pub status: String, // "pending", "in_progress", "completed", "cancelled"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<String>, // "low", "medium", "high"
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeCodeTodoItem {
    pub content: String,
    pub status: String, // "pending", "in_progress", "completed"
    pub active_form: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TodoFile {
    pub session_id: String,
    pub items: Vec<TodoItem>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StopOutput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision: Option<String>, // "block", "continue"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_hook_active: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_mode: Option<PermissionMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inject_prompt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreCompactOutput {
    #[serde(flatten)]
    pub common: HookCommonOutput,
    
    /// Additional context to inject into compaction prompt
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<Vec<String>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hook_specific_output: Option<PreCompactHookSpecificOutput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreCompactHookSpecificOutput {
    pub hook_event_name: String, // "PreCompact"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_context: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClaudeCodeContent {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "tool_use")]
    ToolUse { id: String, name: String, input: HashMap<String, serde_json::Value> },
    #[serde(rename = "tool_result")]
    ToolResult { tool_use_id: String, content: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeCodeMessage {
    #[serde(rename = "type")]
    pub message_type: String, // "user", "assistant"
    pub message: ClaudeCodeMessageInner,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeCodeMessageInner {
    pub role: String, // "user", "assistant"
    pub content: Vec<ClaudeCodeContent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled_hooks: Option<DisabledHooks>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keyword_detector_disabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DisabledHooks {
    All(bool),
    Events(Vec<ClaudeHookEvent>),
}