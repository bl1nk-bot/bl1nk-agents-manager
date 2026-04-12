use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PermissionValue {
    Ask,
    Allow,
    Deny,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BashPermission {
    Value(PermissionValue),
    Map(HashMap<String, PermissionValue>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPermission {
    pub edit: Option<PermissionValue>,
    pub bash: Option<BashPermission>,
    pub webfetch: Option<PermissionValue>,
    pub task: Option<PermissionValue>,
    pub doom_loop: Option<PermissionValue>,
    pub external_directory: Option<PermissionValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum BuiltinAgentName {
    Sisyphus,
    Hephaestus,
    Prometheus,
    Oracle,
    Librarian,
    Explore,
    #[serde(rename = "multimodal-looker")]
    MultimodalLooker,
    Metis,
    Momus,
    Atlas,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum BuiltinSkillName {
    Playwright,
    #[serde(rename = "agent-browser")]
    AgentBrowser,
    #[serde(rename = "dev-browser")]
    DevBrowser,
    #[serde(rename = "frontend-ui-ux")]
    FrontendUiUx,
    #[serde(rename = "git-master")]
    GitMaster,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OverridableAgentName {
    Build,
    Plan,
    Sisyphus,
    Hephaestus,
    #[serde(rename = "sisyphus-junior")]
    SisyphusJunior,
    #[serde(rename = "OpenCode-Builder")]
    OpenCodeBuilder,
    Prometheus,
    Metis,
    Momus,
    Oracle,
    Librarian,
    Explore,
    #[serde(rename = "multimodal-looker")]
    MultimodalLooker,
    Atlas,
}

pub type AgentName = BuiltinAgentName;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum HookName {
    #[serde(rename = "todo-continuation-enforcer")]
    TodoContinuationEnforcer,
    #[serde(rename = "context-window-monitor")]
    ContextWindowMonitor,
    #[serde(rename = "session-recovery")]
    SessionRecovery,
    #[serde(rename = "session-notification")]
    SessionNotification,
    #[serde(rename = "comment-checker")]
    CommentChecker,
    #[serde(rename = "grep-output-truncator")]
    GrepOutputTruncator,
    #[serde(rename = "tool-output-truncator")]
    ToolOutputTruncator,
    #[serde(rename = "question-label-truncator")]
    QuestionLabelTruncator,
    #[serde(rename = "directory-agents-injector")]
    DirectoryAgentsInjector,
    #[serde(rename = "directory-readme-injector")]
    DirectoryReadmeInjector,
    #[serde(rename = "empty-task-response-detector")]
    EmptyTaskResponseDetector,
    #[serde(rename = "think-mode")]
    ThinkMode,
    #[serde(rename = "subagent-question-blocker")]
    SubagentQuestionBlocker,
    #[serde(rename = "anthropic-context-window-limit-recovery")]
    AnthropicContextWindowLimitRecovery,
    #[serde(rename = "preemptive-compaction")]
    PreemptiveCompaction,
    #[serde(rename = "rules-injector")]
    RulesInjector,
    #[serde(rename = "background-notification")]
    BackgroundNotification,
    #[serde(rename = "auto-update-checker")]
    AutoUpdateChecker,
    #[serde(rename = "startup-toast")]
    StartupToast,
    #[serde(rename = "keyword-detector")]
    KeywordDetector,
    #[serde(rename = "agent-usage-reminder")]
    AgentUsageReminder,
    #[serde(rename = "non-interactive-env")]
    NonInteractiveEnv,
    #[serde(rename = "interactive-bash-session")]
    InteractiveBashSession,
    #[serde(rename = "thinking-block-validator")]
    ThinkingBlockValidator,
    #[serde(rename = "ralph-loop")]
    RalphLoop,
    #[serde(rename = "category-skill-reminder")]
    CategorySkillReminder,
    #[serde(rename = "compaction-context-injector")]
    CompactionContextInjector,
    #[serde(rename = "compaction-todo-preserver")]
    CompactionTodoPreserver,
    #[serde(rename = "claude-code-hooks")]
    ClaudeCodeHooks,
    #[serde(rename = "auto-slash-command")]
    AutoSlashCommand,
    #[serde(rename = "edit-error-recovery")]
    EditErrorRecovery,
    #[serde(rename = "delegate-task-retry")]
    DelegateTaskRetry,
    #[serde(rename = "prometheus-md-only")]
    PrometheusMdOnly,
    #[serde(rename = "sisyphus-junior-notepad")]
    SisyphusJuniorNotepad,
    #[serde(rename = "start-work")]
    StartWork,
    Atlas,
    #[serde(rename = "unstable-agent-babysitter")]
    UnstableAgentBabysitter,
    #[serde(rename = "task-reminder")]
    TaskReminder,
    #[serde(rename = "task-resume-info")]
    TaskResumeInfo,
    #[serde(rename = "stop-continuation-guard")]
    StopContinuationGuard,
    #[serde(rename = "tasks-todowrite-disabler")]
    TasksTodowriteDisabler,
    #[serde(rename = "write-existing-file-guard")]
    WriteExistingFileGuard,
    #[serde(rename = "anthropic-effort")]
    AnthropicEffort,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum BuiltinCommandName {
    #[serde(rename = "init-deep")]
    InitDeep,
    #[serde(rename = "ralph-loop")]
    RalphLoop,
    #[serde(rename = "ulw-loop")]
    UlwLoop,
    #[serde(rename = "cancel-ralph")]
    CancelRalph,
    Refactor,
    #[serde(rename = "start-work")]
    StartWork,
    #[serde(rename = "stop-continuation")]
    StopContinuation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentMode {
    Subagent,
    Primary,
    All,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThinkingType {
    Enabled,
    Disabled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingConfig {
    #[serde(rename = "type")]
    pub thinking_type: ThinkingType,
    #[serde(rename = "budgetTokens")]
    pub budget_tokens: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReasoningEffort {
    Low,
    Medium,
    High,
    #[serde(rename = "xhigh")]
    XHigh,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TextVerbosity {
    Low,
    Medium,
    High,
}

pub use crate::agents::types::{AgentOverrideConfig, AgentOverrides};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeCodeConfig {
    pub mcp: Option<bool>,
    pub commands: Option<bool>,
    pub skills: Option<bool>,
    pub agents: Option<bool>,
    pub hooks: Option<bool>,
    pub plugins: Option<bool>,
    pub plugins_override: Option<HashMap<String, bool>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SisyphusAgentConfig {
    pub disabled: Option<bool>,
    pub default_builder_enabled: Option<bool>,
    pub planner_enabled: Option<bool>,
    pub replace_plan: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryConfig {
    pub description: Option<String>,
    pub model: Option<String>,
    pub variant: Option<String>,
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    #[serde(rename = "maxTokens")]
    pub max_tokens: Option<u64>,
    pub thinking: Option<ThinkingConfig>,
    pub reasoning_effort: Option<ReasoningEffort>,
    pub text_verbosity: Option<TextVerbosity>,
    pub tools: Option<HashMap<String, bool>>,
    pub prompt_append: Option<String>,
    pub is_unstable_agent: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BuiltinCategoryName {
    #[serde(rename = "visual-engineering")]
    VisualEngineering,
    Ultrabrain,
    Deep,
    Artistry,
    Quick,
    #[serde(rename = "unspecified-low")]
    UnspecifiedLow,
    #[serde(rename = "unspecified-high")]
    UnspecifiedHigh,
    Writing,
}

pub type CategoriesConfig = HashMap<String, CategoryConfig>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentCheckerConfig {
    pub custom_prompt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicContextPruningTurnProtection {
    pub enabled: Option<bool>,
    pub turns: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicContextPruningDeduplication {
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicContextPruningSupersedeWrites {
    pub enabled: Option<bool>,
    pub aggressive: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicContextPruningPurgeErrors {
    pub enabled: Option<bool>,
    pub turns: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicContextPruningStrategies {
    pub deduplication: Option<DynamicContextPruningDeduplication>,
    pub supersede_writes: Option<DynamicContextPruningSupersedeWrites>,
    pub purge_errors: Option<DynamicContextPruningPurgeErrors>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicContextPruningConfig {
    pub enabled: Option<bool>,
    pub notification: Option<String>,
    pub turn_protection: Option<DynamicContextPruningTurnProtection>,
    pub protected_tools: Option<Vec<String>>,
    pub strategies: Option<DynamicContextPruningStrategies>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentalConfig {
    pub aggressive_truncation: Option<bool>,
    pub auto_resume: Option<bool>,
    pub preemptive_compaction: Option<bool>,
    pub truncate_all_tool_outputs: Option<bool>,
    pub dynamic_context_pruning: Option<DynamicContextPruningConfig>,
    pub task_system: Option<bool>,
    pub plugin_load_timeout_ms: Option<u64>,
    pub safe_hook_creation: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SkillSource {
    Path(String),
    Object {
        path: String,
        recursive: Option<bool>,
        glob: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDefinition {
    pub description: Option<String>,
    pub template: Option<String>,
    pub from: Option<String>,
    pub model: Option<String>,
    pub agent: Option<String>,
    pub subtask: Option<bool>,
    #[serde(rename = "argument-hint")]
    pub argument_hint: Option<String>,
    pub license: Option<String>,
    pub compatibility: Option<String>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    #[serde(rename = "allowed-tools")]
    pub allowed_tools: Option<Vec<String>>,
    pub disable: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SkillEntry {
    Bool(bool),
    Definition(SkillDefinition),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillsConfigMap {
    #[serde(flatten)]
    pub entries: HashMap<String, SkillEntry>,
    pub sources: Option<Vec<SkillSource>>,
    pub enable: Option<Vec<String>>,
    pub disable: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SkillsConfig {
    List(Vec<String>),
    Map(SkillsConfigMap),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RalphLoopConfig {
    pub enabled: Option<bool>,
    pub default_max_iterations: Option<u32>,
    pub state_dir: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundTaskConfig {
    #[serde(rename = "defaultConcurrency")]
    pub default_concurrency: Option<u32>,
    #[serde(rename = "providerConcurrency")]
    pub provider_concurrency: Option<HashMap<String, u32>>,
    #[serde(rename = "modelConcurrency")]
    pub model_concurrency: Option<HashMap<String, u32>>,
    #[serde(rename = "staleTimeoutMs")]
    pub stale_timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    pub force_enable: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BabysittingConfig {
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitMasterConfig {
    pub commit_footer: Option<serde_json::Value>,
    pub include_co_authored_by: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BrowserAutomationProvider {
    Playwright,
    #[serde(rename = "agent-browser")]
    AgentBrowser,
    #[serde(rename = "dev-browser")]
    DevBrowser,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserAutomationConfig {
    pub provider: Option<BrowserAutomationProvider>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WebsearchProvider {
    Exa,
    Tavily,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebsearchConfig {
    pub provider: Option<WebsearchProvider>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TmuxLayout {
    #[serde(rename = "main-horizontal")]
    MainHorizontal,
    #[serde(rename = "main-vertical")]
    MainVertical,
    Tiled,
    #[serde(rename = "even-horizontal")]
    EvenHorizontal,
    #[serde(rename = "even-vertical")]
    EvenVertical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmuxConfig {
    pub enabled: Option<bool>,
    pub layout: Option<TmuxLayout>,
    pub main_pane_size: Option<u32>,
    pub main_pane_min_width: Option<u32>,
    pub agent_pane_min_width: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SisyphusTasksConfig {
    pub storage_path: Option<String>,
    pub task_list_id: Option<String>,
    pub claude_code_compat: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SisyphusConfig {
    pub tasks: Option<SisyphusTasksConfig>,
}

pub type AnyMcpName = String;
pub type McpName = String;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Bl1nkConfig {
    #[serde(rename = "$schema")]
    pub schema: Option<String>,
    pub new_task_system_enabled: Option<bool>,
    pub default_run_agent: Option<String>,
    pub disabled_mcps: Option<Vec<AnyMcpName>>,
    pub disabled_agents: Option<Vec<BuiltinAgentName>>,
    pub disabled_skills: Option<Vec<BuiltinSkillName>>,
    pub disabled_hooks: Option<Vec<HookName>>,
    pub disabled_commands: Option<Vec<BuiltinCommandName>>,
    pub disabled_tools: Option<Vec<String>>,
    #[serde(default)]
    pub agents: AgentOverrides,
    pub categories: Option<CategoriesConfig>,
    pub claude_code: Option<ClaudeCodeConfig>,
    pub sisyphus_agent: Option<SisyphusAgentConfig>,
    pub comment_checker: Option<CommentCheckerConfig>,
    pub experimental: Option<ExperimentalConfig>,
    pub auto_update: Option<bool>,
    pub skills: Option<SkillsConfig>,
    pub ralph_loop: Option<RalphLoopConfig>,
    pub background_task: Option<BackgroundTaskConfig>,
    pub notification: Option<NotificationConfig>,
    pub babysitting: Option<BabysittingConfig>,
    pub git_master: Option<GitMasterConfig>,
    pub browser_automation_engine: Option<BrowserAutomationConfig>,
    pub websearch: Option<WebsearchConfig>,
    pub tmux: Option<TmuxConfig>,
    pub sisyphus: Option<SisyphusConfig>,
    #[serde(rename = "_migrations")]
    pub migrations: Option<Vec<String>>,
}
