use anyhow::Result;

// Module declarations
pub mod adapters;
pub mod agents;
pub mod cli;
pub mod config;
pub mod events;
pub mod filesystem;
pub mod features;
pub mod mcp;
pub mod projects;
pub mod rate_limit;
pub mod rpc;
pub mod search;
pub mod session;

// Test utilities (only available in test builds)
#[cfg(test)]
pub mod test_utils;

// Re-exports
pub use adapters::acp::{
    AuthenticateParams, ContentBlock, InitializeParams, InitializeResult, Location,
    PermissionOutcome, PermissionResult, SessionNewParams, SessionNewResult, SessionPromptParams,
    SessionPromptResult, SessionRequestPermissionParams, SessionUpdate, SessionUpdateParams,
    ToolCallContentItem, ToolCallKind, ToolCallStatus,
};
pub use cli::{AssistantChunk, CommandResult, MessageChunk, StreamAssistantMessageChunkParams};
pub use events::{
    CliIoPayload, CliIoType, ErrorPayload, EventEmitter, GeminiOutputPayload, GeminiThoughtPayload,
    InternalEvent, ToolCallConfirmation, ToolCallConfirmationContent, ToolCallConfirmationRequest,
    ToolCallEvent, ToolCallLocation, ToolCallUpdate,
};
pub use filesystem::{DirEntry, FileContent, GeminiCommand, GitInfo, VolumeType};
pub use projects::{
    ensure_project_metadata, list_enriched_projects, list_projects, make_enriched_project,
    maybe_touch_updated_at, EnrichedProject, ProjectListItem, ProjectMetadata, ProjectMetadataView,
    ProjectsResponse, TouchThrottle,
};
pub use rpc::{JsonRpcError, JsonRpcRequest, JsonRpcResponse, RpcLogger};
pub use search::{
    ConversationHistoryEntry, DetailedConversation, MessageMatch, RecentChat, SearchFilters,
    SearchResult,
};
pub use session::{
    initialize_session, GeminiAuthConfig, LLxprtConfig, PersistentSession, ProcessStatus,
    QwenConfig, SessionManager, SessionParams,
};

pub fn get_session_manager() -> std::sync::Arc<session::SessionManager> {
    session::SessionManager::get_instance()
}

// Standard library imports
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// Main backend interface for Gemini CLI functionality
pub struct GeminiBackend<E: EventEmitter + 'static> {
    emitter: E,
    session_manager: Arc<SessionManager>,
}

impl<E: EventEmitter + 'static> GeminiBackend<E> {
    /// Create a new GeminiBackend instance
    pub fn new(emitter: E) -> Self {
        Self {
            emitter,
            session_manager: SessionManager::get_instance(),
        }
    }

    pub async fn list_projects(&self, limit: u32, offset: u32) -> Result<serde_json::Value> {
        let response: ProjectsResponse = projects::list_projects(limit, offset)?;
        Ok(serde_json::to_value(response)?)
    }

    pub async fn list_enriched_projects(&self) -> Result<Vec<EnrichedProject>> {
        projects::list_enriched_projects()
    }

    pub async fn get_enriched_project(
        &self,
        sha256: String,
        external_root_path: String,
    ) -> Result<EnrichedProject> {
        projects::get_enriched_project(sha256, external_root_path).await
    }

    pub async fn get_project_discussions(&self, project_id: &str) -> Result<Vec<RecentChat>> {
        search::get_project_discussions(project_id).await
    }

    pub async fn get_recent_chats(&self) -> Result<Vec<RecentChat>> {
        search::get_recent_chats().await
    }

    pub async fn search_chats(
        &self,
        query: String,
        filters: Option<SearchFilters>,
    ) -> Result<Vec<SearchResult>> {
        search::search_chats(query, filters).await
    }

    pub async fn get_detailed_conversation(&self, chat_id: &str) -> Result<DetailedConversation> {
        search::get_detailed_conversation(chat_id).await
    }

    pub async fn export_conversation_history(&self, chat_id: &str, format: &str) -> Result<String> {
        search::export_conversation_history(chat_id, format).await
    }

    pub async fn check_cli_installed(&self) -> Result<bool> {
        self.check_cli_installed_named("gemini").await
    }

    pub async fn check_cli_installed_named(&self, name: &str) -> Result<bool> {
        let output = std::process::Command::new(name).arg("--version").output();
        Ok(output.map(|o| o.status.success()).unwrap_or(false))
    }

    pub async fn initialize_session(
        &self,
        session_id: String,
        working_directory: String,
        model: String,
        cli_name: Option<String>,
        backend_config: Option<QwenConfig>,
        gemini_auth: Option<GeminiAuthConfig>,
        llxprt_config: Option<LLxprtConfig>,
    ) -> Result<()> {
        let params = SessionParams {
            session_id,
            working_directory,
            model,
            cli_name,
            backend_config,
            gemini_auth,
            llxprt_config,
        };

        session::initialize_session(params, self.emitter.clone(), &self.session_manager).await?;
        Ok(())
    }

    pub fn approve_oauth(&self) {
        session::approve_oauth();
    }

    pub async fn send_message(
        &self,
        session_id: String,
        message: String,
        conversation_history: String,
    ) -> Result<()> {
        let processes = self.session_manager.get_processes();

        let (sender, rpc_logger, acp_session_id) = {
            let processes_guard = processes
                .lock()
                .map_err(|_| anyhow::anyhow!("Failed to lock processes mutex"))?;
            let session = processes_guard
                .get(&session_id)
                .ok_or_else(|| anyhow::anyhow!("Session not found: {session_id}"))?;

            let sender = session
                .message_sender
                .clone()
                .ok_or_else(|| anyhow::anyhow!("Session not ready for messaging: {session_id}"))?;

            (
                sender,
                session.rpc_logger.clone(),
                session.acp_session_id.clone().unwrap_or(session_id.clone()),
            )
        };

        let mut prompt_text = String::new();
        if !conversation_history.trim().is_empty() {
            prompt_text.push_str("Conversation history:\n");
            prompt_text.push_str(&conversation_history);
            prompt_text.push_str("\n\n");
        }
        prompt_text.push_str(&message);

        let params = SessionPromptParams {
            session_id: acp_session_id,
            prompt: vec![ContentBlock::Text { text: prompt_text }],
        };

        let request_id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| (d.as_millis() % (u32::MAX as u128)) as u32)
            .unwrap_or(1);

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: request_id,
            method: "session/prompt".to_string(),
            params: serde_json::to_value(params)?,
        };

        let request_json = serde_json::to_string(&request)?;
        let _ = rpc_logger.log_rpc(&request_json);
        sender
            .send(request_json)
            .map_err(|e| anyhow::anyhow!("Failed to send prompt: {e}"))?;

        Ok(())
    }

    pub fn get_process_statuses(&self) -> Result<Vec<ProcessStatus>> {
        self.session_manager.get_process_statuses()
    }

    pub fn kill_process(&self, conversation_id: &str) -> Result<()> {
        self.session_manager.kill_process(conversation_id)
    }

    pub async fn handle_tool_confirmation(
        &self,
        session_id: String,
        request_id: u32,
        tool_call_id: String,
        outcome: String,
    ) -> Result<()> {
        let result = serde_json::json!({
            "tool_call_id": tool_call_id,
            "outcome": outcome,
        });

        session::send_response_to_cli(
            &session_id,
            request_id,
            Some(result),
            None::<rpc::JsonRpcError>,
            self.session_manager.get_processes(),
        )
        .await;

        Ok(())
    }

    pub async fn execute_confirmed_command(&self, command: String) -> Result<String> {
        let output = if cfg!(windows) {
            tokio::process::Command::new("cmd")
                .args(["/C", &command])
                .output()
                .await?
        } else {
            tokio::process::Command::new("bash")
                .args(["-lc", &command])
                .output()
                .await?
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if output.status.success() {
            Ok(stdout.trim().to_string())
        } else {
            Err(anyhow::anyhow!("Command failed: {}", stderr.trim()))
        }
    }

    pub async fn generate_conversation_title(
        &self,
        message: String,
        _model: Option<String>,
    ) -> Result<String> {
        let mut title = message.trim().to_string();
        if title.chars().count() > 50 {
            title = title.chars().take(50).collect::<String>() + "...";
        }
        if title.is_empty() {
            Ok("Chat Session".to_string())
        } else {
            Ok(title)
        }
    }

    pub async fn validate_directory(&self, path: String) -> Result<bool> {
        filesystem::validate_directory(path).await
    }

    pub async fn is_home_directory(&self, path: String) -> Result<bool> {
        filesystem::is_home_directory(path).await
    }

    pub async fn get_home_directory(&self) -> Result<String> {
        filesystem::get_home_directory().await
    }

    pub async fn get_parent_directory(&self, path: String) -> Result<Option<String>> {
        filesystem::get_parent_directory(path).await
    }

    pub async fn list_directory_contents(&self, path: String) -> Result<Vec<DirEntry>> {
        filesystem::list_directory_contents(path).await
    }

    pub async fn list_files_recursive(&self, path: String) -> Result<Vec<DirEntry>> {
        filesystem::list_files_recursive(path).await
    }

    pub async fn list_volumes(&self) -> Result<Vec<DirEntry>> {
        filesystem::list_volumes().await
    }

    pub async fn list_gemini_commands(
        &self,
        working_directory: String,
    ) -> Result<Vec<GeminiCommand>> {
        filesystem::list_gemini_commands(working_directory).await
    }

    pub async fn get_git_info(&self, directory: String) -> Result<Option<GitInfo>> {
        filesystem::get_git_info(directory).await
    }

    pub async fn read_file_content(&self, path: String) -> Result<FileContent> {
        filesystem::read_file_content(path).await
    }

    pub async fn read_binary_file_as_base64(&self, path: String) -> Result<String> {
        filesystem::read_binary_file_as_base64(path).await
    }

    pub async fn read_file_content_with_options(
        &self,
        path: String,
        force_text: bool,
    ) -> Result<FileContent> {
        filesystem::read_file_content_with_options(path, force_text).await
    }

    pub async fn write_file_content(&self, path: String, content: String) -> Result<FileContent> {
        filesystem::write_file_content(path, content).await
    }

    pub async fn get_canonical_path(&self, path: String) -> Result<String> {
        let canonical = std::fs::canonicalize(path)?;
        Ok(canonical.to_string_lossy().to_string())
    }
}
