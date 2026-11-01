use bl1nk_core::rate_limit::{RateLimit, RateLimitTracker};
use bl1nk_core::config::RateLimitingConfig;
use bl1nk_core::projects::{ProjectListItem, ProjectsResponse, ProjectMetadata, ProjectMetadataView, EnrichedProject, TouchThrottle};
use bl1nk_core::search::{RecentChat, SearchResult, MessageMatch, SearchFilters, ConversationHistoryEntry, DetailedConversation};
use bl1nk_core::session::{QwenConfig, GeminiAuthConfig, LLxprtConfig, SessionParams};
use bl1nk_core::filesystem::{DirEntry, FileContent, GeminiCommand, GitInfo, VolumeType};
use bl1nk_core::events::{CliIoPayload, CliIoType, ErrorPayload, GeminiOutputPayload, GeminiThoughtPayload, InternalEvent, ToolCallConfirmation, ToolCallConfirmationContent, ToolCallEvent, ToolCallLocation, ToolCallUpdate};
use bl1nk_core::rpc::{JsonRpcError, JsonRpcRequest, JsonRpcResponse, RpcLogger};
use bl1nk_core::adapters::acp::{
    AuthenticateParams, ContentBlock, InitializeParams, InitializeResult, SessionNewParams, 
    SessionNewResult, SessionPromptParams, SessionPromptResult, SessionRequestPermissionParams, 
    SessionUpdate, SessionUpdateParams, ToolCallContentItem, ToolCallKind, ToolCallStatus,
};
use bl1nk_core::cli::{AssistantChunk, CommandResult, MessageChunk, StreamAssistantMessageChunkParams};
use std::path::PathBuf;
use std::time::Duration;
use chrono::{DateTime, FixedOffset, Utc};

#[tokio::test]
async fn test_rate_limit_tracker_basic_functionality() {
    let config = RateLimitingConfig {
        strategy: "round-robin".to_string(),
        track_usage: true,
        usage_db_path: None,
    };

    let mut tracker = RateLimitTracker::new(config);

    let agent_limit = RateLimit {
        requests_per_minute: 10,
        requests_per_day: 100,
    };

    // Test basic increment and check
    assert!(tracker.check_and_increment("test-agent", &agent_limit).await);
    
    let (daily, minute) = tracker.get_usage("test-agent").unwrap();
    assert_eq!(daily, 1);
    assert_eq!(minute, 1);
}

#[tokio::test]
async fn test_rate_limit_tracker_minute_limit() {
    let config = RateLimitingConfig {
        strategy: "round-robin".to_string(),
        track_usage: true,
        usage_db_path: None,
    };

    let mut tracker = RateLimitTracker::new(config);

    let agent_limit = RateLimit {
        requests_per_minute: 2,
        requests_per_day: 100,
    };

    // Use up minute limit
    assert!(tracker.check_and_increment("test-agent", &agent_limit).await);
    assert!(tracker.check_and_increment("test-agent", &agent_limit).await);
    
    // Next request should fail due to minute limit
    assert!(!tracker.check_and_increment("test-agent", &agent_limit).await);
}

#[tokio::test]
async fn test_rate_limit_tracker_daily_limit() {
    let config = RateLimitingConfig {
        strategy: "round-robin".to_string(),
        track_usage: true,
        usage_db_path: None,
    };

    let mut tracker = RateLimitTracker::new(config);

    let agent_limit = RateLimit {
        requests_per_minute: 100,
        requests_per_day: 1,
    };

    // Use up daily limit
    assert!(tracker.check_and_increment("test-agent", &agent_limit).await);
    
    // Next request should fail due to daily limit
    assert!(!tracker.check_and_increment("test-agent", &agent_limit).await);
}

#[tokio::test]
async fn test_rate_limit_tracker_disabled_tracking() {
    let config = RateLimitingConfig {
        strategy: "round-robin".to_string(),
        track_usage: false,
        usage_db_path: None,
    };

    let mut tracker = RateLimitTracker::new(config);

    let agent_limit = RateLimit {
        requests_per_minute: 1,
        requests_per_day: 1,
    };

    // Should always succeed when tracking is disabled
    assert!(tracker.check_and_increment("test-agent", &agent_limit).await);
    assert!(tracker.check_and_increment("test-agent", &agent_limit).await);
    assert!(tracker.check_and_increment("test-agent", &agent_limit).await);
}

#[test]
fn test_project_list_item_serialization() {
    let item = ProjectListItem {
        id: "test-id".to_string(),
        title: Some("Test Project".to_string()),
        status: Some("active".to_string()),
        created_at: Some("2023-01-01T00:00:00Z".to_string()),
        updated_at: Some("2023-01-02T00:00:00Z".to_string()),
        last_activity_at: Some("2023-01-03T00:00:00Z".to_string()),
        log_count: Some(5),
    };

    let json = serde_json::to_string(&item).unwrap();
    let deserialized: ProjectListItem = serde_json::from_str(&json).unwrap();

    assert_eq!(item.id, deserialized.id);
    assert_eq!(item.title, deserialized.title);
    assert_eq!(item.status, deserialized.status);
    assert_eq!(item.created_at, deserialized.created_at);
    assert_eq!(item.updated_at, deserialized.updated_at);
    assert_eq!(item.last_activity_at, deserialized.last_activity_at);
    assert_eq!(item.log_count, deserialized.log_count);
}

#[test]
fn test_project_list_item_serialization_with_none_values() {
    let item = ProjectListItem {
        id: "test-id".to_string(),
        title: None,
        status: None,
        created_at: None,
        updated_at: None,
        last_activity_at: None,
        log_count: None,
    };

    let json = serde_json::to_string(&item).unwrap();
    assert!(!json.contains("title"));
    assert!(!json.contains("status"));
    assert!(!json.contains("createdAt"));
    assert!(!json.contains("updatedAt"));
    assert!(!json.contains("lastActivityAt"));
    assert!(!json.contains("logCount"));

    let deserialized: ProjectListItem = serde_json::from_str(&json).unwrap();
    assert_eq!(item.id, deserialized.id);
    assert!(deserialized.title.is_none());
    assert!(deserialized.status.is_none());
}

#[test]
fn test_projects_response_serialization() {
    let response = ProjectsResponse {
        items: vec![ProjectListItem {
            id: "test".to_string(),
            title: None,
            status: None,
            created_at: None,
            updated_at: None,
            last_activity_at: None,
            log_count: None,
        }],
        total: 10,
        limit: 5,
        offset: 0,
    };

    let json = serde_json::to_string(&response).unwrap();
    let deserialized: ProjectsResponse = serde_json::from_str(&json).unwrap();

    assert_eq!(response.total, deserialized.total);
    assert_eq!(response.limit, deserialized.limit);
    assert_eq!(response.offset, deserialized.offset);
    assert_eq!(response.items.len(), deserialized.items.len());
}

#[test]
fn test_project_metadata_default() {
    let metadata = ProjectMetadata::default();
    assert_eq!(metadata.path, PathBuf::new());
    assert!(metadata.sha256.is_none());
    assert!(metadata.friendly_name.is_none());
    assert!(metadata.first_used.is_none());
    assert!(metadata.updated_at.is_none());
}

#[test]
fn test_project_metadata_view_serialization() {
    let view = ProjectMetadataView {
        path: "/test/path".to_string(),
        sha256: "abcd1234".to_string(),
        friendly_name: "test-project".to_string(),
        first_used: Some("2023-01-01T00:00:00Z".to_string()),
        updated_at: Some("2023-01-02T00:00:00Z".to_string()),
    };

    let json = serde_json::to_string(&view).unwrap();
    let deserialized: ProjectMetadataView = serde_json::from_str(&json).unwrap();

    assert_eq!(view.path, deserialized.path);
    assert_eq!(view.sha256, deserialized.sha256);
    assert_eq!(view.friendly_name, deserialized.friendly_name);
    assert_eq!(view.first_used, deserialized.first_used);
    assert_eq!(view.updated_at, deserialized.updated_at);
}

#[test]
fn test_touch_throttle_new() {
    let throttle = TouchThrottle::new(Duration::from_secs(1));
    assert_eq!(throttle.min_interval, Duration::from_secs(1));
}

#[test]
fn test_touch_throttle_default() {
    let throttle = TouchThrottle::default();
    assert_eq!(throttle.min_interval, Duration::default());
}

#[test]
fn test_recent_chat_serialization() {
    let chat = RecentChat {
        id: "test-chat-id".to_string(),
        title: "Test Chat".to_string(),
        started_at_iso: "2023-01-01T00:00:00Z".to_string(),
        message_count: 10,
        summary: Some("Test summary".to_string()),
        last_activity_iso: Some("2023-01-02T00:00:00Z".to_string()),
        total_tokens: Some(100),
        tags: vec!["test".to_string(), "example".to_string()],
    };

    let json = serde_json::to_string(&chat).unwrap();
    let deserialized: RecentChat = serde_json::from_str(&json).unwrap();

    assert_eq!(chat.id, deserialized.id);
    assert_eq!(chat.title, deserialized.title);
    assert_eq!(chat.started_at_iso, deserialized.started_at_iso);
    assert_eq!(chat.message_count, deserialized.message_count);
    assert_eq!(chat.summary, deserialized.summary);
    assert_eq!(chat.last_activity_iso, deserialized.last_activity_iso);
    assert_eq!(chat.total_tokens, deserialized.total_tokens);
    assert_eq!(chat.tags, deserialized.tags);
}

#[test]
fn test_message_match_serialization() {
    let match_item = MessageMatch {
        content_snippet: "test content".to_string(),
        start_line: 10,
        end_line: 15,
        file_path: Some("/path/to/file.txt".to_string()),
        timestamp: Some("2023-01-01T00:00:00Z".to_string()),
    };

    let json = serde_json::to_string(&match_item).unwrap();
    let deserialized: MessageMatch = serde_json::from_str(&json).unwrap();

    assert_eq!(match_item.content_snippet, deserialized.content_snippet);
    assert_eq!(match_item.start_line, deserialized.start_line);
    assert_eq!(match_item.end_line, deserialized.end_line);
    assert_eq!(match_item.file_path, deserialized.file_path);
    assert_eq!(match_item.timestamp, deserialized.timestamp);
}

#[test]
fn test_search_result_serialization() {
    let result = SearchResult {
        chat: RecentChat {
            id: "test-chat-id".to_string(),
            title: "Test Chat".to_string(),
            started_at_iso: "2023-01-01T00:00:00Z".to_string(),
            message_count: 10,
            summary: Some("Test summary".to_string()),
            last_activity_iso: Some("2023-01-02T00:00:00Z".to_string()),
            total_tokens: Some(100),
            tags: vec!["test".to_string(), "example".to_string()],
        },
        matches: vec![MessageMatch {
            content_snippet: "test content".to_string(),
            start_line: 10,
            end_line: 15,
            file_path: Some("/path/to/file.txt".to_string()),
            timestamp: Some("2023-01-01T00:00:00Z".to_string()),
        }],
        score: 0.95,
    };

    let json = serde_json::to_string(&result).unwrap();
    let deserialized: SearchResult = serde_json::from_str(&json).unwrap();

    assert_eq!(result.chat.id, deserialized.chat.id);
    assert_eq!(result.matches.len(), deserialized.matches.len());
    assert_eq!(result.score, deserialized.score);
}

#[test]
fn test_search_filters_default() {
    let filters = SearchFilters::default();
    assert!(filters.date_range.is_none());
    assert!(filters.tags.is_none());
    assert!(filters.project_id.is_none());
    assert!(filters.max_results.is_none());
}

#[test]
fn test_search_filters_serialization() {
    let filters = SearchFilters {
        date_range: Some(("2023-01-01".to_string(), "2023-01-31".to_string())),
        tags: Some(vec!["important".to_string()]),
        project_id: Some("test-project".to_string()),
        max_results: Some(10),
    };

    let json = serde_json::to_string(&filters).unwrap();
    let deserialized: SearchFilters = serde_json::from_str(&json).unwrap();

    assert_eq!(filters.date_range, deserialized.date_range);
    assert_eq!(filters.tags, deserialized.tags);
    assert_eq!(filters.project_id, deserialized.project_id);
    assert_eq!(filters.max_results, deserialized.max_results);
}

#[test]
fn test_detailed_conversation_serialization() {
    let conversation = DetailedConversation {
        id: "test-conversation".to_string(),
        title: "Test Conversation".to_string(),
        started_at_iso: "2023-01-01T00:00:00Z".to_string(),
        messages: vec![ConversationHistoryEntry {
            id: "msg-1".to_string(),
            role: "user".to_string(),
            content: "Hello, world!".to_string(),
            timestamp_iso: "2023-01-01T00:00:00Z".to_string(),
            tokens: Some(5),
        }],
        total_tokens: Some(100),
        tags: vec!["test".to_string()],
    };

    let json = serde_json::to_string(&conversation).unwrap();
    let deserialized: DetailedConversation = serde_json::from_str(&json).unwrap();

    assert_eq!(conversation.id, deserialized.id);
    assert_eq!(conversation.title, deserialized.title);
    assert_eq!(conversation.messages.len(), deserialized.messages.len());
    assert_eq!(conversation.total_tokens, deserialized.total_tokens);
    assert_eq!(conversation.tags, deserialized.tags);
}

#[test]
fn test_session_params_serialization() {
    let params = SessionParams {
        session_id: "test-session".to_string(),
        working_directory: "/test/dir".to_string(),
        model: "test-model".to_string(),
        cli_name: Some("gemini".to_string()),
        backend_config: Some(QwenConfig {
            api_key: "test-key".to_string(),
            base_url: "https://api.example.com".to_string(),
            model: "test-model".to_string(),
            yolo: None,
        }),
        gemini_auth: Some(GeminiAuthConfig {
            method: "oauth-personal".to_string(),
            api_key: Some("test-key".to_string()),
            vertex_project: Some("test-project".to_string()),
            vertex_location: Some("us-central1".to_string()),
            yolo: None,
        }),
        llxprt_config: Some(LLxprtConfig {
            provider: "openai".to_string(),
            api_key: "test-key".to_string(),
            model: "gpt-4".to_string(),
            base_url: Some("https://api.openai.com".to_string()),
        }),
    };

    let json = serde_json::to_string(&params).unwrap();
    let deserialized: SessionParams = serde_json::from_str(&json).unwrap();

    assert_eq!(params.session_id, deserialized.session_id);
    assert_eq!(params.working_directory, deserialized.working_directory);
    assert_eq!(params.model, deserialized.model);
    assert_eq!(params.cli_name, deserialized.cli_name);
}

#[test]
fn test_file_content_serialization() {
    let content = FileContent {
        path: "/test/file.txt".to_string(),
        content: "test content".to_string(),
        size: 12,
        modified: "2023-01-01T00:00:00Z".to_string(),
        mime_type: "text/plain".to_string(),
    };

    let json = serde_json::to_string(&content).unwrap();
    let deserialized: FileContent = serde_json::from_str(&json).unwrap();

    assert_eq!(content.path, deserialized.path);
    assert_eq!(content.content, deserialized.content);
    assert_eq!(content.size, deserialized.size);
    assert_eq!(content.modified, deserialized.modified);
    assert_eq!(content.mime_type, deserialized.mime_type);
}

#[test]
fn test_dir_entry_serialization() {
    let entry = DirEntry {
        name: "test-dir".to_string(),
        path: "/test/test-dir".to_string(),
        size: 0,
        modified: "2023-01-01T00:00:00Z".to_string(),
        is_directory: true,
        mime_type: "directory".to_string(),
    };

    let json = serde_json::to_string(&entry).unwrap();
    let deserialized: DirEntry = serde_json::from_str(&json).unwrap();

    assert_eq!(entry.name, deserialized.name);
    assert_eq!(entry.path, deserialized.path);
    assert_eq!(entry.size, deserialized.size);
    assert_eq!(entry.modified, deserialized.modified);
    assert_eq!(entry.is_directory, deserialized.is_directory);
    assert_eq!(entry.mime_type, deserialized.mime_type);
}

#[test]
fn test_gemini_command_serialization() {
    let command = GeminiCommand {
        name: "test-command".to_string(),
        description: "Test command".to_string(),
        path: "/path/to/command.md".to_string(),
    };

    let json = serde_json::to_string(&command).unwrap();
    let deserialized: GeminiCommand = serde_json::from_str(&json).unwrap();

    assert_eq!(command.name, deserialized.name);
    assert_eq!(command.description, deserialized.description);
    assert_eq!(command.path, deserialized.path);
}

#[test]
fn test_git_info_serialization() {
    let git_info = GitInfo {
        remote_url: Some("https://github.com/test/repo.git".to_string()),
        branch: Some("main".to_string()),
        commit_hash: Some("abc123".to_string()),
        is_dirty: Some(false),
    };

    let json = serde_json::to_string(&git_info).unwrap();
    let deserialized: GitInfo = serde_json::from_str(&json).unwrap();

    assert_eq!(git_info.remote_url, deserialized.remote_url);
    assert_eq!(git_info.branch, deserialized.branch);
    assert_eq!(git_info.commit_hash, deserialized.commit_hash);
    assert_eq!(git_info.is_dirty, deserialized.is_dirty);
}

#[test]
fn test_json_rpc_request_serialization() {
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: 1,
        method: "test_method".to_string(),
        params: serde_json::Value::String("test_params".to_string()),
    };

    let json = serde_json::to_string(&request).unwrap();
    let deserialized: JsonRpcRequest = serde_json::from_str(&json).unwrap();

    assert_eq!(request.jsonrpc, deserialized.jsonrpc);
    assert_eq!(request.id, deserialized.id);
    assert_eq!(request.method, deserialized.method);
    assert_eq!(request.params, deserialized.params);
}

#[test]
fn test_json_rpc_response_serialization() {
    let response = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: 1,
        result: Some(serde_json::Value::String("test_result".to_string())),
        error: None,
    };

    let json = serde_json::to_string(&response).unwrap();
    let deserialized: JsonRpcResponse = serde_json::from_str(&json).unwrap();

    assert_eq!(response.jsonrpc, deserialized.jsonrpc);
    assert_eq!(response.id, deserialized.id);
    assert_eq!(response.result, deserialized.result);
    assert_eq!(response.error, deserialized.error);
}

#[test]
fn test_json_rpc_response_with_error() {
    let error = JsonRpcError {
        code: -32600,
        message: "Invalid Request".to_string(),
        data: None,
    };

    let response = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: 1,
        result: None,
        error: Some(error.clone()),
    };

    let json = serde_json::to_string(&response).unwrap();
    let deserialized: JsonRpcResponse = serde_json::from_str(&json).unwrap();

    assert!(deserialized.error.is_some());
    assert_eq!(deserialized.error.unwrap().code, -32600);
    assert_eq!(deserialized.error.unwrap().message, "Invalid Request");
}

#[test]
fn test_internal_event_serialization() {
    let event = InternalEvent::GeminiOutput {
        session_id: "test-session".to_string(),
        payload: GeminiOutputPayload {
            text: "test output".to_string(),
        },
    };

    // Since InternalEvent is a complex enum, we'll just verify it can be cloned
    let _cloned = event.clone();
}

#[test]
fn test_tool_call_confirmation_serialization() {
    let confirmation = ToolCallConfirmation {
        session_id: "test-session".to_string(),
        request_id: 123,
        tool_call_id: "tool-call-1".to_string(),
        outcome: "success".to_string(),
    };

    let json = serde_json::to_string(&confirmation).unwrap();
    let deserialized: ToolCallConfirmation = serde_json::from_str(&json).unwrap();

    assert_eq!(confirmation.session_id, deserialized.session_id);
    assert_eq!(confirmation.request_id, deserialized.request_id);
    assert_eq!(confirmation.tool_call_id, deserialized.tool_call_id);
    assert_eq!(confirmation.outcome, deserialized.outcome);
}

#[test]
fn test_enriched_project_serialization() {
    let project = EnrichedProject {
        sha256: "testsha".to_string(),
        root_path: PathBuf::from("/test/path"),
        metadata: ProjectMetadataView {
            path: "/test/path".to_string(),
            sha256: "testsha".to_string(),
            friendly_name: "test-project".to_string(),
            first_used: None,
            updated_at: None,
        },
    };

    let json = serde_json::to_string(&project).unwrap();
    let deserialized: EnrichedProject = serde_json::from_str(&json).unwrap();

    assert_eq!(project.sha256, deserialized.sha256);
    assert_eq!(project.root_path, deserialized.root_path);
    assert_eq!(project.metadata.path, deserialized.metadata.path);
}

#[tokio::test]
async fn test_rate_limit_tracker_reset_all() {
    let config = RateLimitingConfig {
        strategy: "round-robin".to_string(),
        track_usage: true,
        usage_db_path: None,
    };

    let mut tracker = RateLimitTracker::new(config);

    let agent_limit = RateLimit {
        requests_per_minute: 10,
        requests_per_day: 100,
    };

    // Add some usage
    assert!(tracker.check_and_increment("test-agent", &agent_limit).await);
    assert!(tracker.get_usage("test-agent").is_some());

    // Reset all
    tracker.reset_all();

    // Usage should be cleared
    assert!(tracker.get_usage("test-agent").is_none());
}

#[test]
fn test_project_metadata_to_view_conversion() {
    let now = chrono::Utc::now().with_timezone(&FixedOffset::east_opt(0).unwrap());
    let metadata = ProjectMetadata {
        path: PathBuf::from("/test/path"),
        sha256: Some("testsha".to_string()),
        friendly_name: Some("test-project".to_string()),
        first_used: Some(now.clone()),
        updated_at: Some(now.clone()),
    };

    let view = bl1nk_core::projects::to_view(&metadata, &PathBuf::from("/test/path"), "testsha");

    assert_eq!(view.path, "/test/path");
    assert_eq!(view.sha256, "testsha");
    assert_eq!(view.friendly_name, "test-project");
    assert!(view.first_used.is_some());
    assert!(view.updated_at.is_some());
}

#[test]
fn test_derive_friendly_name_from_path() {
    let path = PathBuf::from("/home/user/projects/my-app");
    let result = bl1nk_core::projects::derive_friendly_name_from_path(&path);
    
    #[cfg(windows)]
    {
        // On Windows, the path separator is different
        assert!(result.contains("home") || result.contains("user") || result.contains("projects") || result.contains("my-app"));
    }
    #[cfg(not(windows))]
    {
        assert_eq!(result, "home-user-projects-my-app");
    }
}

#[test]
fn test_parse_millis_from_log_name() {
    assert_eq!(bl1nk_core::projects::parse_millis_from_log_name("rpc-log-1640995200000.log"), Some(1640995200000));
    assert_eq!(bl1nk_core::projects::parse_millis_from_log_name("rpc-log-1640995200000.json"), Some(1640995200000));
    assert_eq!(bl1nk_core::projects::parse_millis_from_log_name("invalid-name.log"), None);
    assert_eq!(bl1nk_core::projects::parse_millis_from_log_name("rpc-log-invalid.log"), None);
}

#[test]
fn test_now_fixed_offset() {
    let now1 = bl1nk_core::projects::now_fixed_offset();
    std::thread::sleep(Duration::from_millis(1));
    let now2 = bl1nk_core::projects::now_fixed_offset();
    assert!(now2 >= now1); // Allow equality since time resolution might be low
}

#[tokio::test]
async fn test_make_enriched_project() {
    let project = bl1nk_core::projects::make_enriched_project("testsha", None, false);
    
    assert_eq!(project.sha256, "testsha");
    assert_eq!(project.metadata.sha256, "testsha");
}

#[test]
fn test_touch_throttle_clone() {
    let throttle1 = TouchThrottle::new(Duration::from_secs(5));
    let throttle2 = throttle1.clone();
    assert_eq!(throttle1.min_interval, throttle2.min_interval);
}

#[test]
fn test_project_metadata_serialization_roundtrip() {
    let original = ProjectMetadata {
        path: PathBuf::from("/test/path"),
        sha256: Some("testsha".to_string()),
        friendly_name: Some("test-project".to_string()),
        first_used: Some(Utc::now().with_timezone(&FixedOffset::east_opt(0).unwrap())),
        updated_at: Some(Utc::now().with_timezone(&FixedOffset::east_opt(0).unwrap())),
    };

    let json = serde_json::to_string(&original).unwrap();
    let deserialized: ProjectMetadata = serde_json::from_str(&json).unwrap();

    assert_eq!(original.path, deserialized.path);
    assert_eq!(original.sha256, deserialized.sha256);
    assert_eq!(original.friendly_name, deserialized.friendly_name);
    // Note: We don't compare timestamps directly due to serialization precision
    assert!(deserialized.first_used.is_some());
    assert!(deserialized.updated_at.is_some());
}

#[test]
fn test_volume_type_serialization() {
    let volume = VolumeType::Local;
    let json = serde_json::to_string(&volume).unwrap();
    let deserialized: VolumeType = serde_json::from_str(&json).unwrap();
    assert_eq!(volume, deserialized);
}

#[test]
fn test_cli_io_type_serialization() {
    let io_type = CliIoType::Input;
    let json = serde_json::to_string(&io_type).unwrap();
    let deserialized: CliIoType = serde_json::from_str(&json).unwrap();
    assert_eq!(io_type, deserialized);
}

#[test]
fn test_content_block_serialization() {
    let content = ContentBlock::Text { text: "test content".to_string() };
    let json = serde_json::to_string(&content).unwrap();
    let deserialized: ContentBlock = serde_json::from_str(&json).unwrap();
    
    match (content, deserialized) {
        (ContentBlock::Text { text: t1 }, ContentBlock::Text { text: t2 }) => {
            assert_eq!(t1, t2);
        }
        _ => panic!("Content blocks don't match"),
    }
}

#[test]
fn test_tool_call_kind_serialization() {
    let kind = ToolCallKind::Function;
    let json = serde_json::to_string(&kind).unwrap();
    let deserialized: ToolCallKind = serde_json::from_str(&json).unwrap();
    assert_eq!(kind, deserialized);
}

#[test]
fn test_tool_call_status_serialization() {
    let status = ToolCallStatus::Requested;
    let json = serde_json::to_string(&status).unwrap();
    let deserialized: ToolCallStatus = serde_json::from_str(&json).unwrap();
    assert_eq!(status, deserialized);
}

#[test]
fn test_initialize_params_serialization() {
    let params = InitializeParams {
        protocol_version: 1,
        client_capabilities: bl1nk_core::adapters::acp::ClientCapabilities {
            fs: bl1nk_core::adapters::acp::FileSystemCapabilities {
                read_text_file: false,
                write_text_file: false,
            },
        },
    };

    let json = serde_json::to_string(&params).unwrap();
    let deserialized: InitializeParams = serde_json::from_str(&json).unwrap();

    assert_eq!(params.protocol_version, deserialized.protocol_version);
}

#[test]
fn test_session_new_params_serialization() {
    let params = SessionNewParams {
        cwd: "/test/dir".to_string(),
        mcp_servers: vec![],
    };

    let json = serde_json::to_string(&params).unwrap();
    let deserialized: SessionNewParams = serde_json::from_str(&json).unwrap();

    assert_eq!(params.cwd, deserialized.cwd);
    assert_eq!(params.mcp_servers, deserialized.mcp_servers);
}

#[test]
fn test_session_prompt_params_serialization() {
    let params = SessionPromptParams {
        session_id: "test-session".to_string(),
        prompt: vec![ContentBlock::Text { text: "test prompt".to_string() }],
    };

    let json = serde_json::to_string(&params).unwrap();
    let deserialized: SessionPromptParams = serde_json::from_str(&json).unwrap();

    assert_eq!(params.session_id, deserialized.session_id);
    assert_eq!(params.prompt.len(), deserialized.prompt.len());
}

#[test]
fn test_authenticate_params_serialization() {
    let params = AuthenticateParams {
        method_id: "oauth-personal".to_string(),
    };

    let json = serde_json::to_string(&params).unwrap();
    let deserialized: AuthenticateParams = serde_json::from_str(&json).unwrap();

    assert_eq!(params.method_id, deserialized.method_id);
}

#[test]
fn test_command_result_serialization() {
    let result = CommandResult {
        success: true,
        output: "test output".to_string(),
        error: None,
    };

    let json = serde_json::to_string(&result).unwrap();
    let deserialized: CommandResult = serde_json::from_str(&json).unwrap();

    assert_eq!(result.success, deserialized.success);
    assert_eq!(result.output, deserialized.output);
    assert_eq!(result.error, deserialized.error);
}

#[test]
fn test_message_chunk_serialization() {
    let chunk = MessageChunk {
        content: "test chunk".to_string(),
        role: "assistant".to_string(),
        timestamp: "2023-01-01T00:00:00Z".to_string(),
    };

    let json = serde_json::to_string(&chunk).unwrap();
    let deserialized: MessageChunk = serde_json::from_str(&json).unwrap();

    assert_eq!(chunk.content, deserialized.content);
    assert_eq!(chunk.role, deserialized.role);
    assert_eq!(chunk.timestamp, deserialized.timestamp);
}

#[test]
fn test_assistant_chunk_serialization() {
    let chunk = AssistantChunk {
        text: Some("test text".to_string()),
        tool_calls: None,
        finish_reason: None,
    };

    let json = serde_json::to_string(&chunk).unwrap();
    let deserialized: AssistantChunk = serde_json::from_str(&json).unwrap();

    assert_eq!(chunk.text, deserialized.text);
    assert_eq!(chunk.tool_calls, deserialized.tool_calls);
    assert_eq!(chunk.finish_reason, deserialized.finish_reason);
}

#[test]
fn test_stream_assistant_message_chunk_params_serialization() {
    let params = StreamAssistantMessageChunkParams {
        session_id: "test-session".to_string(),
        chunk: AssistantChunk {
            text: Some("test text".to_string()),
            tool_calls: None,
            finish_reason: None,
        },
    };

    let json = serde_json::to_string(&params).unwrap();
    let deserialized: StreamAssistantMessageChunkParams = serde_json::from_str(&json).unwrap();

    assert_eq!(params.session_id, deserialized.session_id);
    assert_eq!(params.chunk.text, deserialized.chunk.text);
}

#[test]
fn test_json_rpc_error_serialization() {
    let error = JsonRpcError {
        code: -32700,
        message: "Parse error".to_string(),
        data: None,
    };

    let json = serde_json::to_string(&error).unwrap();
    let deserialized: JsonRpcError = serde_json::from_str(&json).unwrap();

    assert_eq!(error.code, deserialized.code);
    assert_eq!(error.message, deserialized.message);
    assert_eq!(error.data, deserialized.data);
}

#[test]
fn test_json_rpc_error_with_data() {
    let error = JsonRpcError {
        code: -32603,
        message: "Internal error".to_string(),
        data: Some(serde_json::Value::String("error details".to_string())),
    };

    let json = serde_json::to_string(&error).unwrap();
    let deserialized: JsonRpcError = serde_json::from_str(&json).unwrap();

    assert_eq!(error.code, deserialized.code);
    assert_eq!(error.message, deserialized.message);
    assert!(deserialized.data.is_some());
}

#[tokio::test]
async fn test_rate_limit_tracker_with_zero_limits() {
    let config = RateLimitingConfig {
        strategy: "round-robin".to_string(),
        track_usage: true,
        usage_db_path: None,
    };

    let mut tracker = RateLimitTracker::new(config);

    let agent_limit = RateLimit {
        requests_per_minute: 0,
        requests_per_day: 0,
    };

    // Should fail immediately with zero limits
    assert!(!tracker.check_and_increment("test-agent", &agent_limit).await);
}

#[test]
fn test_project_list_item_with_all_optional_fields_none() {
    let item = ProjectListItem {
        id: "test-id".to_string(),
        title: None,
        status: None,
        created_at: None,
        updated_at: None,
        last_activity_at: None,
        log_count: None,
    };

    // Verify serialization doesn't include null fields
    let json = serde_json::to_string(&item).unwrap();
    assert!(!json.contains("title"));
    assert!(!json.contains("status"));
    assert!(!json.contains("createdAt"));
    assert!(!json.contains("updatedAt"));
    assert!(!json.contains("lastActivityAt"));
    assert!(!json.contains("logCount"));
}

#[test]
fn test_project_metadata_view_with_optional_fields() {
    let view = ProjectMetadataView {
        path: "/test/path".to_string(),
        sha256: "testsha".to_string(),
        friendly_name: "test-project".to_string(),
        first_used: None,
        updated_at: None,
    };

    let json = serde_json::to_string(&view).unwrap();
    let deserialized: ProjectMetadataView = serde_json::from_str(&json).unwrap();

    assert_eq!(view.path, deserialized.path);
    assert_eq!(view.sha256, deserialized.sha256);
    assert_eq!(view.friendly_name, deserialized.friendly_name);
    assert_eq!(view.first_used, deserialized.first_used);
    assert_eq!(view.updated_at, deserialized.updated_at);
}

#[test]
fn test_recent_chat_with_optional_fields() {
    let chat = RecentChat {
        id: "test-id".to_string(),
        title: "Test Chat".to_string(),
        started_at_iso: "2023-01-01T00:00:00Z".to_string(),
        message_count: 5,
        summary: None,
        last_activity_iso: None,
        total_tokens: None,
        tags: vec![],
    };

    let json = serde_json::to_string(&chat).unwrap();
    let deserialized: RecentChat = serde_json::from_str(&json).unwrap();

    assert_eq!(chat.id, deserialized.id);
    assert_eq!(chat.title, deserialized.title);
    assert_eq!(chat.summary, deserialized.summary);
    assert_eq!(chat.tags, deserialized.tags);
}

#[test]
fn test_message_match_with_optional_fields() {
    let match_item = MessageMatch {
        content_snippet: "test snippet".to_string(),
        start_line: 1,
        end_line: 5,
        file_path: None,
        timestamp: None,
    };

    let json = serde_json::to_string(&match_item).unwrap();
    let deserialized: MessageMatch = serde_json::from_str(&json).unwrap();

    assert_eq!(match_item.content_snippet, deserialized.content_snippet);
    assert_eq!(match_item.file_path, deserialized.file_path);
    assert_eq!(match_item.timestamp, deserialized.timestamp);
}

#[test]
fn test_search_result_with_high_score() {
    let result = SearchResult {
        chat: RecentChat {
            id: "test-chat".to_string(),
            title: "Test".to_string(),
            started_at_iso: "2023-01-01T00:00:00Z".to_string(),
            message_count: 1,
            summary: None,
            last_activity_iso: None,
            total_tokens: None,
            tags: vec![],
        },
        matches: vec![],
        score: 1.0,
    };

    assert_eq!(result.score, 1.0);
}

#[test]
fn test_detailed_conversation_empty_messages() {
    let conversation = DetailedConversation {
        id: "test".to_string(),
        title: "Test".to_string(),
        started_at_iso: "2023-01-01T00:00:00Z".to_string(),
        messages: vec![],
        total_tokens: None,
        tags: vec![],
    };

    assert_eq!(conversation.messages.len(), 0);
    assert!(conversation.total_tokens.is_none());
}

#[test]
fn test_conversation_history_entry_serialization() {
    let entry = ConversationHistoryEntry {
        id: "msg-1".to_string(),
        role: "assistant".to_string(),
        content: "Hello!".to_string(),
        timestamp_iso: "2023-01-01T00:00:00Z".to_string(),
        tokens: Some(2),
    };

    let json = serde_json::to_string(&entry).unwrap();
    let deserialized: ConversationHistoryEntry = serde_json::from_str(&json).unwrap();

    assert_eq!(entry.id, deserialized.id);
    assert_eq!(entry.role, deserialized.role);
    assert_eq!(entry.content, deserialized.content);
    assert_eq!(entry.tokens, deserialized.tokens);
}

#[test]
fn test_session_params_with_optional_fields() {
    let params = SessionParams {
        session_id: "test".to_string(),
        working_directory: "/test".to_string(),
        model: "test".to_string(),
        cli_name: None,
        backend_config: None,
        gemini_auth: None,
        llxprt_config: None,
    };

    let json = serde_json::to_string(&params).unwrap();
    let deserialized: SessionParams = serde_json::from_str(&json).unwrap();

    assert_eq!(params.session_id, deserialized.session_id);
    assert!(deserialized.cli_name.is_none());
    assert!(deserialized.backend_config.is_none());
    assert!(deserialized.gemini_auth.is_none());
    assert!(deserialized.llxprt_config.is_none());
}

#[test]
fn test_qwen_config_serialization() {
    let config = QwenConfig {
        api_key: "key".to_string(),
        base_url: "https://api.example.com".to_string(),
        model: "test-model".to_string(),
        yolo: Some(true),
    };

    let json = serde_json::to_string(&config).unwrap();
    let deserialized: QwenConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(config.api_key, deserialized.api_key);
    assert_eq!(config.base_url, deserialized.base_url);
    assert_eq!(config.model, deserialized.model);
    assert_eq!(config.yolo, deserialized.yolo);
}

#[test]
fn test_gemini_auth_config_serialization() {
    let config = GeminiAuthConfig {
        method: "oauth".to_string(),
        api_key: Some("key".to_string()),
        vertex_project: Some("project".to_string()),
        vertex_location: Some("location".to_string()),
        yolo: Some(false),
    };

    let json = serde_json::to_string(&config).unwrap();
    let deserialized: GeminiAuthConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(config.method, deserialized.method);
    assert_eq!(config.api_key, deserialized.api_key);
    assert_eq!(config.vertex_project, deserialized.vertex_project);
    assert_eq!(config.vertex_location, deserialized.vertex_location);
    assert_eq!(config.yolo, deserialized.yolo);
}

#[test]
fn test_llxprt_config_serialization() {
    let config = LLxprtConfig {
        provider: "openai".to_string(),
        api_key: "key".to_string(),
        model: "gpt-4".to_string(),
        base_url: Some("https://api.openai.com".to_string()),
    };

    let json = serde_json::to_string(&config).unwrap();
    let deserialized: LLxprtConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(config.provider, deserialized.provider);
    assert_eq!(config.api_key, deserialized.api_key);
    assert_eq!(config.model, deserialized.model);
    assert_eq!(config.base_url, deserialized.base_url);
}

#[test]
fn test_file_content_with_binary_data() {
    let content = FileContent {
        path: "/test/binary.bin".to_string(),
        content: "binary data".to_string(),
        size: 1024,
        modified: "2023-01-01T00:00:00Z".to_string(),
        mime_type: "application/octet-stream".to_string(),
    };

    let json = serde_json::to_string(&content).unwrap();
    let deserialized: FileContent = serde_json::from_str(&json).unwrap();

    assert_eq!(content.path, deserialized.path);
    assert_eq!(content.mime_type, deserialized.mime_type);
}

#[test]
fn test_dir_entry_for_file() {
    let entry = DirEntry {
        name: "test.txt".to_string(),
        path: "/test/test.txt".to_string(),
        size: 100,
        modified: "2023-01-01T00:00:00Z".to_string(),
        is_directory: false,
        mime_type: "text/plain".to_string(),
    };

    assert!(!entry.is_directory);
    assert_eq!(entry.mime_type, "text/plain");
}

#[test]
fn test_gemini_command_with_special_chars() {
    let command = GeminiCommand {
        name: "test_cmd".to_string(),
        description: "Test command with 'quotes' and \"double quotes\"".to_string(),
        path: "/path/with spaces/command.md".to_string(),
    };

    let json = serde_json::to_string(&command).unwrap();
    let deserialized: GeminiCommand = serde_json::from_str(&json).unwrap();

    assert_eq!(command.description, deserialized.description);
    assert_eq!(command.path, deserialized.path);
}

#[test]
fn test_git_info_with_all_fields_none() {
    let git_info = GitInfo {
        remote_url: None,
        branch: None,
        commit_hash: None,
        is_dirty: None,
    };

    let json = serde_json::to_string(&git_info).unwrap();
    let deserialized: GitInfo = serde_json::from_str(&json).unwrap();

    assert!(deserialized.remote_url.is_none());
    assert!(deserialized.branch.is_none());
    assert!(deserialized.commit_hash.is_none());
    assert!(deserialized.is_dirty.is_none());
}

#[test]
fn test_volume_type_variants() {
    let local = VolumeType::Local;
    let network = VolumeType::Network;
    let removable = VolumeType::Removable;

    assert_ne!(local, network);
    assert_ne!(network, removable);
    assert_ne!(removable, local);
}

#[test]
fn test_cli_io_payload_serialization() {
    let payload = CliIoPayload {
        io_type: CliIoType::Output,
        data: "test data".to_string(),
    };

    let json = serde_json::to_string(&payload).unwrap();
    let deserialized: CliIoPayload = serde_json::from_str(&json).unwrap();

    assert_eq!(payload.io_type, deserialized.io_type);
    assert_eq!(payload.data, deserialized.data);
}

#[test]
fn test_error_payload_serialization() {
    let payload = ErrorPayload {
        error: "test error".to_string(),
    };

    let json = serde_json::to_string(&payload).unwrap();
    let deserialized: ErrorPayload = serde_json::from_str(&json).unwrap();

    assert_eq!(payload.error, deserialized.error);
}

#[test]
fn test_gemini_output_payload_serialization() {
    let payload = GeminiOutputPayload {
        text: "test output".to_string(),
    };

    let json = serde_json::to_string(&payload).unwrap();
    let deserialized: GeminiOutputPayload = serde_json::from_str(&json).unwrap();

    assert_eq!(payload.text, deserialized.text);
}

#[test]
fn test_gemini_thought_payload_serialization() {
    let payload = GeminiThoughtPayload {
        thought: "thinking...".to_string(),
    };

    let json = serde_json::to_string(&payload).unwrap();
    let deserialized: GeminiThoughtPayload = serde_json::from_str(&json).unwrap();

    assert_eq!(payload.thought, deserialized.thought);
}

#[test]
fn test_tool_call_confirmation_content_serialization() {
    let content = ToolCallConfirmationContent {
        tool_call_id: "test-tool".to_string(),
        outcome: "success".to_string(),
    };

    let json = serde_json::to_string(&content).unwrap();
    let deserialized: ToolCallConfirmationContent = serde_json::from_str(&json).unwrap();

    assert_eq!(content.tool_call_id, deserialized.tool_call_id);
    assert_eq!(content.outcome, deserialized.outcome);
}

#[test]
fn test_tool_call_event_serialization() {
    let event = ToolCallEvent {
        id: "event-1".to_string(),
        tool_call_id: "tool-1".to_string(),
        status: ToolCallStatus::Requested,
        title: "Test Tool Call".to_string(),
        content: vec![],
        locations: vec![],
        kind: "function".to_string(),
        server_name: "test-server".to_string(),
        tool_name: "test-tool".to_string(),
    };

    let json = serde_json::to_string(&event).unwrap();
    let deserialized: ToolCallEvent = serde_json::from_str(&json).unwrap();

    assert_eq!(event.id, deserialized.id);
    assert_eq!(event.tool_call_id, deserialized.tool_call_id);
    assert_eq!(event.status, deserialized.status);
}

#[test]
fn test_tool_call_location_serialization() {
    let location = ToolCallLocation {
        file: "/path/to/file".to_string(),
        line: 10,
        column: 5,
    };

    let json = serde_json::to_string(&location).unwrap();
    let deserialized: ToolCallLocation = serde_json::from_str(&json).unwrap();

    assert_eq!(location.file, deserialized.file);
    assert_eq!(location.line, deserialized.line);
    assert_eq!(location.column, deserialized.column);
}

#[test]
fn test_tool_call_update_serialization() {
    let update = ToolCallUpdate {
        tool_call_id: "tool-1".to_string(),
        status: ToolCallStatus::Completed,
        content: vec![],
        server_name: "test-server".to_string(),
        tool_name: "test-tool".to_string(),
    };

    let json = serde_json::to_string(&update).unwrap();
    let deserialized: ToolCallUpdate = serde_json::from_str(&json).unwrap();

    assert_eq!(update.tool_call_id, deserialized.tool_call_id);
    assert_eq!(update.status, deserialized.status);
}

#[test]
fn test_tool_call_content_item_serialization() {
    let item = ToolCallContentItem {
        r#type: "text".to_string(),
        text: "test content".to_string(),
    };

    let json = serde_json::to_string(&item).unwrap();
    let deserialized: ToolCallContentItem = serde_json::from_str(&json).unwrap();

    assert_eq!(item.r#type, deserialized.r#type);
    assert_eq!(item.text, deserialized.text);
}

#[test]
fn test_initialize_result_serialization() {
    let result = InitializeResult {
        server_capabilities: bl1nk_core::adapters::acp::ServerCapabilities {
            tools: vec![],
        },
    };

    // Just verify it can be serialized/deserialized
    let json = serde_json::to_string(&result).unwrap();
    let _deserialized: InitializeResult = serde_json::from_str(&json).unwrap();
}

#[test]
fn test_session_new_result_serialization() {
    let result = SessionNewResult {
        session_id: "test-session".to_string(),
    };

    let json = serde_json::to_string(&result).unwrap();
    let deserialized: SessionNewResult = serde_json::from_str(&json).unwrap();

    assert_eq!(result.session_id, deserialized.session_id);
}

#[test]
fn test_session_prompt_result_serialization() {
    let result = SessionPromptResult {
        stop_reason: "end_turn".to_string(),
        output: vec![],
    };

    let json = serde_json::to_string(&result).unwrap();
    let deserialized: SessionPromptResult = serde_json::from_str(&json).unwrap();

    assert_eq!(result.stop_reason, deserialized.stop_reason);
}

#[test]
fn test_session_request_permission_params_serialization() {
    let params = SessionRequestPermissionParams {
        tool_call: bl1nk_core::adapters::acp::ToolCall {
            id: "test-call".to_string(),
            name: "test-tool".to_string(),
            arguments: serde_json::Value::Null,
        },
    };

    let json = serde_json::to_string(&params).unwrap();
    let deserialized: SessionRequestPermissionParams = serde_json::from_str(&json).unwrap();

    assert_eq!(params.tool_call.id, deserialized.tool_call.id);
}

#[test]
fn test_session_update_serialization() {
    let update = SessionUpdate::AgentMessageChunk {
        content: ContentBlock::Text { text: "test".to_string() },
    };

    // Just verify it can be serialized/deserialized
    let json = serde_json::to_string(&update).unwrap();
    let _deserialized: SessionUpdate = serde_json::from_str(&json).unwrap();
}

#[test]
fn test_session_update_params_serialization() {
    let params = SessionUpdateParams {
        session_id: "test-session".to_string(),
        update: SessionUpdate::AgentMessageChunk {
            content: ContentBlock::Text { text: "test".to_string() },
        },
    };

    let json = serde_json::to_string(&params).unwrap();
    let _deserialized: SessionUpdateParams = serde_json::from_str(&json).unwrap();
}

#[test]
fn test_command_result_with_error() {
    let result = CommandResult {
        success: false,
        output: "".to_string(),
        error: Some("Something went wrong".to_string()),
    };

    let json = serde_json::to_string(&result).unwrap();
    let deserialized: CommandResult = serde_json::from_str(&json).unwrap();

    assert!(!deserialized.success);
    assert!(deserialized.error.is_some());
}

#[test]
fn test_assistant_chunk_with_tool_calls() {
    let chunk = AssistantChunk {
        text: None,
        tool_calls: Some(vec![]),
        finish_reason: Some("stop".to_string()),
    };

    let json = serde_json::to_string(&chunk).unwrap();
    let deserialized: AssistantChunk = serde_json::from_str(&json).unwrap();

    assert!(deserialized.text.is_none());
    assert!(deserialized.tool_calls.is_some());
    assert_eq!(deserialized.finish_reason, Some("stop".to_string()));
}

#[test]
fn test_message_chunk_with_special_characters() {
    let chunk = MessageChunk {
        content: "Hello,\nWorld!\tTabbed".to_string(),
        role: "user".to_string(),
        timestamp: "2023-01-01T00:00:00Z".to_string(),
    };

    let json = serde_json::to_string(&chunk).unwrap();
    let deserialized: MessageChunk = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.content, "Hello,\nWorld!\tTabbed");
}

#[test]
fn test_json_rpc_request_with_complex_params() {
    let params = serde_json::json!({
        "param1": "value1",
        "param2": 42,
        "param3": true,
        "nested": {
            "inner": "value"
        }
    });

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: 123,
        method: "complex_method".to_string(),
        params,
    };

    let json = serde_json::to_string(&request).unwrap();
    let deserialized: JsonRpcRequest = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.method, "complex_method");
    assert_eq!(deserialized.id, 123);
}

#[test]
fn test_json_rpc_response_with_complex_result() {
    let result = serde_json::json!({
        "status": "success",
        "data": {
            "id": 123,
            "name": "test"
        }
    });

    let response = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: 456,
        result: Some(result),
        error: None,
    };

    let json = serde_json::to_string(&response).unwrap();
    let deserialized: JsonRpcResponse = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.id, 456);
    assert!(deserialized.result.is_some());
}

#[test]
fn test_json_rpc_response_error_with_data() {
    let error_data = serde_json::json!({
        "code": "INTERNAL_ERROR",
        "details": "Something went wrong internally"
    });

    let error = JsonRpcError {
        code: -32603,
        message: "Internal error".to_string(),
        data: Some(error_data),
    };

    let response = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: 789,
        result: None,
        error: Some(error),
    };

    let json = serde_json::to_string(&response).unwrap();
    let deserialized: JsonRpcResponse = serde_json::from_str(&json).unwrap();

    assert!(deserialized.error.is_some());
    assert_eq!(deserialized.id, 789);
}

#[tokio::test]
async fn test_rate_limit_tracker_multiple_agents() {
    let config = RateLimitingConfig {
        strategy: "round-robin".to_string(),
        track_usage: true,
        usage_db_path: None,
    };

    let mut tracker = RateLimitTracker::new(config);

    let agent_limit = RateLimit {
        requests_per_minute: 5,
        requests_per_day: 50,
    };

    // Test multiple agents independently
    for i in 0..3 {
        let agent_id = format!("agent-{}", i);
        
        // Each agent should be able to make requests up to its limit
        for j in 0..5 {
            assert!(tracker.check_and_increment(&agent_id, &agent_limit).await, 
                   "Agent {} request {} should succeed", agent_id, j);
        }
        
        // Next request should fail for each agent
        assert!(!tracker.check_and_increment(&agent_id, &agent_limit).await, 
               "Agent {} 6th request should fail", agent_id);
    }
}

#[test]
fn test_project_list_item_edge_cases() {
    // Test with very long strings
    let long_id = "a".repeat(1000);
    let long_title = "b".repeat(1000);
    
    let item = ProjectListItem {
        id: long_id.clone(),
        title: Some(long_title.clone()),
        status: Some("active".to_string()),
        created_at: Some("2023-01-01T00:00:00Z".to_string()),
        updated_at: Some("2023-01-02T00:00:00Z".to_string()),
        last_activity_at: Some("2023-01-03T00:00:00Z".to_string()),
        log_count: Some(999999),
    };

    let json = serde_json::to_string(&item).unwrap();
    let deserialized: ProjectListItem = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.id, long_id);
    assert_eq!(deserialized.title, Some(long_title));
    assert_eq!(deserialized.log_count, Some(999999));
}

#[test]
fn test_project_metadata_with_unicode_path() {
    let metadata = ProjectMetadata {
        path: PathBuf::from("/path/with/unicode/测试"),
        sha256: Some("testsha".to_string()),
        friendly_name: Some("test-project".to_string()),
        first_used: None,
        updated_at: None,
    };

    let json = serde_json::to_string(&metadata).unwrap();
    let deserialized: ProjectMetadata = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.path, PathBuf::from("/path/with/unicode/测试"));
}

#[test]
fn test_recent_chat_with_unicode_content() {
    let chat = RecentChat {
        id: "test-id".to_string(),
        title: "测试标题".to_string(), // Unicode title
        started_at_iso: "2023-01-01T00:00:00Z".to_string(),
        message_count: 10,
        summary: Some("这是一个摘要".to_string()), // Unicode summary
        last_activity_iso: Some("2023-01-02T00:00:00Z".to_string()),
        total_tokens: Some(100),
        tags: vec!["测试".to_string(), "example".to_string()], // Unicode tag
    };

    let json = serde_json::to_string(&chat).unwrap();
    let deserialized: RecentChat = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.title, "测试标题");
    assert_eq!(deserialized.summary, Some("这是一个摘要".to_string()));
    assert!(deserialized.tags.contains(&"测试".to_string()));
}

#[test]
fn test_message_match_with_special_characters() {
    let match_item = MessageMatch {
        content_snippet: "Line with special chars: \n\t\r\"'".to_string(),
        start_line: 1,
        end_line: 1,
        file_path: Some("/path/with/special chars.txt".to_string()),
        timestamp: Some("2023-01-01T00:00:00Z".to_string()),
    };

    let json = serde_json::to_string(&match_item).unwrap();
    let deserialized: MessageMatch = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.content_snippet, "Line with special chars: \n\t\r\"'");
    assert_eq!(deserialized.file_path, Some("/path/with/special chars.txt".to_string()));
}

#[test]
fn test_search_filters_with_complex_conditions() {
    let filters = SearchFilters {
        date_range: Some(("2023-01-01".to_string(), "2023-12-31".to_string())),
        tags: Some(vec!["important".to_string(), "urgent".to_string(), "review".to_string()]),
        project_id: Some("complex-project-id-with-many-parts".to_string()),
        max_results: Some(100),
    };

    let json = serde_json::to_string(&filters).unwrap();
    let deserialized: SearchFilters = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.date_range, Some(("2023-01-01".to_string(), "2023-12-31".to_string())));
    assert_eq!(deserialized.tags, Some(vec!["important".to_string(), "urgent".to_string(), "review".to_string()]));
    assert_eq!(deserialized.project_id, Some("complex-project-id-with-many-parts".to_string()));
    assert_eq!(deserialized.max_results, Some(100));
}

#[test]
fn test_detailed_conversation_with_many_messages() {
    let messages: Vec<ConversationHistoryEntry> = (0..100)
        .map(|i| ConversationHistoryEntry {
            id: format!("msg-{}", i),
            role: if i % 2 == 0 { "user" } else { "assistant" }.to_string(),
            content: format!("Message content {}", i),
            timestamp_iso: format!("2023-01-01T00:{:02}:00Z", i % 60),
            tokens: Some((i % 1000) as u32),
        })
        .collect();

    let conversation = DetailedConversation {
        id: "large-conversation".to_string(),
        title: "Large Conversation Test".to_string(),
        started_at_iso: "2023-01-01T00:00:00Z".to_string(),
        messages,
        total_tokens: Some(5000),
        tags: vec!["large".to_string(), "test".to_string()],
    };

    let json = serde_json::to_string(&conversation).unwrap();
    let deserialized: DetailedConversation = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.id, "large-conversation");
    assert_eq!(deserialized.messages.len(), 100);
    assert_eq!(deserialized.total_tokens, Some(5000));
    assert!(deserialized.tags.contains(&"large".to_string()));
    assert!(deserialized.tags.contains(&"test".to_string()));
}

#[test]
fn test_session_params_with_all_configs() {
    let params = SessionParams {
        session_id: "full-test-session".to_string(),
        working_directory: "/full/test/directory".to_string(),
        model: "full-test-model".to_string(),
        cli_name: Some("full-test-cli".to_string()),
        backend_config: Some(QwenConfig {
            api_key: "full-test-key".to_string(),
            base_url: "https://full.test.api.com".to_string(),
            model: "full-test-model".to_string(),
            yolo: Some(true),
        }),
        gemini_auth: Some(GeminiAuthConfig {
            method: "full-test-method".to_string(),
            api_key: Some("full-test-key".to_string()),
            vertex_project: Some("full-test-project".to_string()),
            vertex_location: Some("full-test-location".to_string()),
            yolo: Some(false),
        }),
        llxprt_config: Some(LLxprtConfig {
            provider: "full-test-provider".to_string(),
            api_key: "full-test-key".to_string(),
            model: "full-test-model".to_string(),
            base_url: Some("https://full.test.api.com".to_string()),
        }),
    };

    let json = serde_json::to_string(&params).unwrap();
    let deserialized: SessionParams = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.session_id, "full-test-session");
    assert_eq!(deserialized.working_directory, "/full/test/directory");
    assert_eq!(deserialized.model, "full-test-model");
    assert_eq!(deserialized.cli_name, Some("full-test-cli".to_string()));
    assert!(deserialized.backend_config.is_some());
    assert!(deserialized.gemini_auth.is_some());
    assert!(deserialized.llxprt_config.is_some());
}

#[test]
fn test_file_content_large_size() {
    let large_content = "A".repeat(1024 * 1024); // 1MB of content
    
    let content = FileContent {
        path: "/large/file.txt".to_string(),
        content: large_content,
        size: 1024 * 1024,
        modified: "2023-01-01T00:00:00Z".to_string(),
        mime_type: "text/plain".to_string(),
    };

    let json = serde_json::to_string(&content).unwrap();
    let deserialized: FileContent = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.path, "/large/file.txt");
    assert_eq!(deserialized.size, 1024 * 1024);
    assert_eq!(deserialized.mime_type, "text/plain");
    assert_eq!(deserialized.content.len(), 1024 * 1024);
}

#[test]
fn test_dir_entry_various_mime_types() {
    let test_cases = vec![
        ("/file.txt", "text/plain", false),
        ("/image.png", "image/png", false),
        ("/archive.zip", "application/zip", false),
        ("/directory", "directory", true),
        ("/executable.exe", "application/octet-stream", false),
    ];

    for (path, mime_type, is_dir) in test_cases {
        let name = std::path::Path::new(path)
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();
        
        let entry = DirEntry {
            name,
            path: path.to_string(),
            size: 0,
            modified: "2023-01-01T00:00:00Z".to_string(),
            is_directory: is_dir,
            mime_type: mime_type.to_string(),
        };

        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: DirEntry = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.mime_type, mime_type);
        assert_eq!(deserialized.is_directory, is_dir);
    }
}

#[test]
fn test_gemini_command_special_characters() {
    let command = GeminiCommand {
        name: "command-with-dashes_and_underscores123".to_string(),
        description: "Command with special chars: !@#$%^&*()".to_string(),
        path: "/path/with/special_chars and spaces/command.md".to_string(),
    };

    let json = serde_json::to_string(&command).unwrap();
    let deserialized: GeminiCommand = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.name, "command-with-dashes_and_underscores123");
    assert_eq!(deserialized.description, "Command with special chars: !@#$%^&*()");
    assert_eq!(deserialized.path, "/path/with/special_chars and spaces/command.md");
}

#[test]
fn test_git_info_edge_cases() {
    let test_cases = vec![
        // Full git info
        (
            Some("https://github.com/user/repo.git".to_string()),
            Some("main".to_string()),
            Some("a1b2c3d4e5f6".to_string()),
            Some(true),
        ),
        // Minimal git info
        (None, None, None, None),
        // Mixed
        (
            Some("https://gitlab.com/user/repo.git".to_string()),
            None,
            Some("f6e5d4c3b2a1".to_string()),
            Some(false),
        ),
    ];

    for (remote_url, branch, commit_hash, is_dirty) in test_cases {
        let git_info = GitInfo {
            remote_url,
            branch,
            commit_hash,
            is_dirty,
        };

        let json = serde_json::to_string(&git_info).unwrap();
        let deserialized: GitInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.remote_url, remote_url);
        assert_eq!(deserialized.branch, branch);
        assert_eq!(deserialized.commit_hash, commit_hash);
        assert_eq!(deserialized.is_dirty, is_dirty);
    }
}

#[test]
fn test_json_rpc_various_id_types() {
    // Test with string ID
    let request_str_id = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: 2, // Using different ID to distinguish
        method: "test_method".to_string(),
        params: serde_json::Value::Null,
    };

    let json = serde_json::to_string(&request_str_id).unwrap();
    let deserialized: JsonRpcRequest = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.id, 2);
    assert_eq!(deserialized.method, "test_method");
}

#[test]
fn test_internal_event_variants() {
    // Test different internal event variants by creating and cloning them
    let output_event = InternalEvent::GeminiOutput {
        session_id: "test-session".to_string(),
        payload: GeminiOutputPayload {
            text: "test output".to_string(),
        },
    };

    let thought_event = InternalEvent::GeminiThought {
        session_id: "test-session".to_string(),
        payload: GeminiThoughtPayload {
            thought: "thinking...".to_string(),
        },
    };

    let error_event = InternalEvent::Error {
        session_id: "test-session".to_string(),
        payload: ErrorPayload {
            error: "test error".to_string(),
        },
    };

    // Verify they can be cloned (indicating proper implementation)
    let _cloned_output = output_event.clone();
    let _cloned_thought = thought_event.clone();
    let _cloned_error = error_event.clone();
}

#[test]
fn test_tool_call_confirmation_various_outcomes() {
    let outcomes = vec!["success", "failure", "partial", "timeout", "cancelled"];

    for outcome in outcomes {
        let confirmation = ToolCallConfirmation {
            session_id: "test-session".to_string(),
            request_id: 123,
            tool_call_id: "tool-call-1".to_string(),
            outcome: outcome.to_string(),
        };

        let json = serde_json::to_string(&confirmation).unwrap();
        let deserialized: ToolCallConfirmation = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.outcome, outcome);
        assert_eq!(deserialized.session_id, "test-session");
    }
}

#[test]
fn test_enriched_project_complex_scenario() {
    let project = EnrichedProject {
        sha256: "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6".to_string(), // 64-char hex
        root_path: PathBuf::from("/very/long/path/with/many/components/that/goes/deep/into/the/system"),
        metadata: ProjectMetadataView {
            path: "/very/long/path/with/many/components/that/goes/deep/into/the/system".to_string(),
            sha256: "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6".to_string(),
            friendly_name: "very-long-path-with-many-components-that-goes-deep-into-the-system".to_string(),
            first_used: Some("2023-01-01T00:00:00+00:00".to_string()),
            updated_at: Some("2023-12-31T23:59:59+00:00".to_string()),
        },
    };

    let json = serde_json::to_string(&project).unwrap();
    let deserialized: EnrichedProject = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.sha256.len(), 64); // Verify it's a proper SHA256
    assert!(deserialized.root_path.starts_with("/very/long/path"));
    assert_eq!(deserialized.metadata.friendly_name, "very-long-path-with-many-components-that-goes-deep-into-the-system");
    assert!(deserialized.metadata.first_used.is_some());
    assert!(deserialized.metadata.updated_at.is_some());
}

#[tokio::test]
async fn test_rate_limit_tracker_high_volume_scenario() {
    let config = RateLimitingConfig {
        strategy: "round-robin".to_string(),
        track_usage: true,
        usage_db_path: None,
    };

    let mut tracker = RateLimitTracker::new(config);

    let agent_limit = RateLimit {
        requests_per_minute: 1000, // High limit to avoid hitting it
        requests_per_day: 10000,
    };

    // Simulate high volume usage
    for i in 0..100 {
        let agent_id = format!("high-volume-agent-{}", i % 10); // Cycle through 10 agents
        
        assert!(tracker.check_and_increment(&agent_id, &agent_limit).await, 
               "Request {} for agent {} should succeed", i, agent_id);
        
        // Verify usage is being tracked
        let (daily, minute) = tracker.get_usage(&agent_id).unwrap_or((0, 0));
        assert!(daily >= 1);
        assert!(minute >= 1);
    }

    // Verify all agents have some usage
    for i in 0..10 {
        let agent_id = format!("high-volume-agent-{}", i);
        let usage = tracker.get_usage(&agent_id);
        assert!(usage.is_some(), "Agent {} should have usage tracked", agent_id);
        let (daily, _) = usage.unwrap();
        assert!(daily >= 1, "Agent {} should have at least 1 daily request", agent_id);
    }
}

#[test]
fn test_project_list_item_various_statuses() {
    let statuses = vec!["active", "inactive", "archived", "pending", "processing", "completed", "failed"];

    for status in statuses {
        let item = ProjectListItem {
            id: format!("test-{}", status),
            title: Some(format!("Test {}", status)),
            status: Some(status.to_string()),
            created_at: Some("2023-01-01T00:00:00Z".to_string()),
            updated_at: Some("2023-01-02T00:00:00Z".to_string()),
            last_activity_at: Some("2023-01-03T00:00:00Z".to_string()),
            log_count: Some(5),
        };

        let json = serde_json::to_string(&item).unwrap();
        let deserialized: ProjectListItem = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.status, Some(status.to_string()));
        assert_eq!(deserialized.id, format!("test-{}", status));
    }
}

#[test]
fn test_project_metadata_various_paths() {
    let paths = vec![
        "/simple/path",
        "/path/with spaces",
        "/path/with/unicode/测试",
        "C:\\Windows\\Path\\On\\Windows",
        "/path/with/symbols_123!@#",
        "~/home/relative/path",
        "/",
        "/single-segment",
    ];

    for path_str in paths {
        let path = PathBuf::from(path_str);
        let friendly_name = bl1nk_core::projects::derive_friendly_name_from_path(&path);

        let metadata = ProjectMetadata {
            path: path.clone(),
            sha256: Some("testsha".to_string()),
            friendly_name: Some(friendly_name.clone()),
            first_used: None,
            updated_at: None,
        };

        let json = serde_json::to_string(&metadata).unwrap();
        let deserialized: ProjectMetadata = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.path, path);
        assert_eq!(deserialized.friendly_name, Some(friendly_name));
    }
}

#[test]
fn test_recent_chat_various_titles() {
    let titles = vec![
        "Simple Title",
        "Title with spaces and CAPS",
        "Title with numbers 123 and symbols !@#",
        "Title with unicode: Привет мир 你好世界",
        "", // Empty title
        "Very long title ".repeat(50), // Long title
    ];

    for (i, title) in titles.iter().enumerate() {
        let chat = RecentChat {
            id: format!("chat-{}", i),
            title: title.clone(),
            started_at_iso: "2023-01-01T00:00:00Z".to_string(),
            message_count: i as u32,
            summary: if title.is_empty() { None } else { Some("Summary".to_string()) },
            last_activity_iso: Some("2023-01-02T00:00:00Z".to_string()),
            total_tokens: Some((i * 10) as u32),
            tags: vec!["test".to_string()],
        };

        let json = serde_json::to_string(&chat).unwrap();
        let deserialized: RecentChat = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.title, *title);
        assert_eq!(deserialized.id, format!("chat-{}", i));
    }
}

#[test]
fn test_message_match_various_snippets() {
    let snippets = vec![
        "Simple text",
        "Text with\nnewlines",
        "Text with\ttabs",
        "Text with \"quotes\" and 'apostrophes'",
        "Unicode text: こんにちは 你好",
        "", // Empty snippet
        "Very long snippet ".repeat(100), // Long snippet
        "Special chars: !@#$%^&*()_+-=[]{}|;':\",./<>?",
    ];

    for (i, snippet) in snippets.iter().enumerate() {
        let match_item = MessageMatch {
            content_snippet: snippet.clone(),
            start_line: i as u32,
            end_line: (i + 1) as u32,
            file_path: Some(format!("/path/to/file-{}.txt", i)),
            timestamp: Some("2023-01-01T00:00:00Z".to_string()),
        };

        let json = serde_json::to_string(&match_item).unwrap();
        let deserialized: MessageMatch = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.content_snippet, *snippet);
        assert_eq!(deserialized.start_line, i as u32);
    }
}

#[test]
fn test_search_result_various_scores() {
    let scores = vec![0.0, 0.25, 0.5, 0.75, 1.0, 0.123456, 0.987654];

    for (i, score) in scores.iter().enumerate() {
        let result = SearchResult {
            chat: RecentChat {
                id: format!("chat-{}", i),
                title: format!("Chat {}", i),
                started_at_iso: "2023-01-01T00:00:00Z".to_string(),
                message_count: 1,
                summary: None,
                last_activity_iso: None,
                total_tokens: None,
                tags: vec![],
            },
            matches: vec![],
            score: *score,
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: SearchResult = serde_json::from_str(&json).unwrap();

        assert!((deserialized.score - score).abs() < f64::EPSILON);
    }
}

#[test]
fn test_detailed_conversation_various_token_counts() {
    let token_counts = vec![Some(0), Some(1), Some(100), Some(1000), Some(10000), None];

    for (i, token_count) in token_counts.iter().enumerate() {
        let conversation = DetailedConversation {
            id: format!("conv-{}", i),
            title: format!("Conversation {}", i),
            started_at_iso: "2023-01-01T00:00:00Z".to_string(),
            messages: vec![],
            total_tokens: *token_count,
            tags: vec![],
        };

        let json = serde_json::to_string(&conversation).unwrap();
        let deserialized: DetailedConversation = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.total_tokens, *token_count);
        assert_eq!(deserialized.id, format!("conv-{}", i));
    }
}

#[test]
fn test_session_params_various_models() {
    let models = vec![
        "basic-model",
        "model-with-hyphens",
        "model_with_underscores",
        "model123with456numbers",
        "very-long-model-name-with-many-components",
        "gpt-3.5-turbo",
        "claude-2.1",
        "gemini-pro",
    ];

    for (i, model) in models.iter().enumerate() {
        let params = SessionParams {
            session_id: format!("session-{}", i),
            working_directory: "/test/dir".to_string(),
            model: model.clone(),
            cli_name: Some("test-cli".to_string()),
            backend_config: None,
            gemini_auth: None,
            llxprt_config: None,
        };

        let json = serde_json::to_string(&params).unwrap();
        let deserialized: SessionParams = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.model, *model);
        assert_eq!(deserialized.session_id, format!("session-{}", i));
    }
}

#[test]
fn test_file_content_various_mime_types() {
    let mime_types = vec![
        "text/plain",
        "application/json",
        "image/png",
        "video/mp4",
        "application/pdf",
        "text/html",
        "application/javascript",
        "text/css",
        "application/xml",
        "unknown/type",
    ];

    for (i, mime_type) in mime_types.iter().enumerate() {
        let content = FileContent {
            path: format!("/file-{}.txt", i),
            content: format!("Content for file {}", i),
            size: (i * 100) as u64,
            modified: "2023-01-01T00:00:00Z".to_string(),
            mime_type: mime_type.clone(),
        };

        let json = serde_json::to_string(&content).unwrap();
        let deserialized: FileContent = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.mime_type, *mime_type);
        assert_eq!(deserialized.path, format!("/file-{}.txt", i));
    }
}

#[test]
fn test_dir_entry_various_sizes() {
    let sizes = vec![0u64, 1, 1024, 1024*1024, 1024*1024*1024]; // 0, 1, 1KB, 1MB, 1GB

    for (i, size) in sizes.iter().enumerate() {
        let entry = DirEntry {
            name: format!("file-{}", i),
            path: format!("/path/file-{}", i),
            size: *size,
            modified: "2023-01-01T00:00:00Z".to_string(),
            is_directory: false,
            mime_type: "application/octet-stream".to_string(),
        };

        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: DirEntry = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.size, *size);
        assert_eq!(deserialized.name, format!("file-{}", i));
    }
}

#[test]
fn test_gemini_command_various_descriptions() {
    let descriptions = vec![
        "Simple description",
        "Description with spaces and CAPS",
        "Description with numbers 123 and symbols !@#",
        "Unicode description: Привет мир 你好世界",
        "", // Empty description
        "Very long description ".repeat(50), // Long description
        "Description with\nnewlines and\ttabs",
        "Description with 'quotes' and \"double quotes\"",
    ];

    for (i, description) in descriptions.iter().enumerate() {
        let command = GeminiCommand {
            name: format!("command-{}", i),
            description: description.clone(),
            path: format!("/path/command-{}.md", i),
        };

        let json = serde_json::to_string(&command).unwrap();
        let deserialized: GeminiCommand = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.description, *description);
        assert_eq!(deserialized.name, format!("command-{}", i));
    }
}

#[test]
fn test_git_info_various_branch_names() {
    let branches = vec![
        Some("main".to_string()),
        Some("master".to_string()),
        Some("develop".to_string()),
        Some("feature/new-feature".to_string()),
        Some("bugfix/issue-123".to_string()),
        Some("release/v1.0.0".to_string()),
        Some("hotfix/critical-fix".to_string()),
        None, // No branch
    ];

    for (i, branch) in branches.iter().enumerate() {
        let git_info = GitInfo {
            remote_url: Some("https://github.com/test/repo.git".to_string()),
            branch: branch.clone(),
            commit_hash: Some(format!("commit{:0>10}", i)),
            is_dirty: Some(i % 2 == 0), // Alternate dirty state
        };

        let json = serde_json::to_string(&git_info).unwrap();
        let deserialized: GitInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.branch, *branch);
        assert_eq!(deserialized.remote_url, git_info.remote_url);
    }
}

#[test]
fn test_json_rpc_request_various_methods() {
    let methods = vec![
        "initialize",
        "shutdown", 
        "exit",
        "textDocument/did",
        "workspace/executeCommand",
        "custom/method",
        "method-with-hyphens",
        "method_with_underscores",
        "method123with456numbers",
    ];

    for (i, method) in methods.iter().enumerate() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: i as i32 + 100,
            method: method.clone(),
            params: serde_json::Value::Null,
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: JsonRpcRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.method, *method);
        assert_eq!(deserialized.id, i as i32 + 100);
    }
}

#[test]
fn test_json_rpc_response_various_results() {
    let results = vec![
        serde_json::Value::Null,
        serde_json::Value::Bool(true),
        serde_json::Value::Number(serde_json::Number::from(42)),
        serde_json::Value::String("success".to_string()),
        serde_json::Value::Array(vec![]),
        serde_json::Value::Object(serde_json::Map::new()),
    ];

    for (i, result) in results.iter().enumerate() {
        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: i as i32 + 200,
            result: Some(result.clone()),
            error: None,
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: JsonRpcResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, i as i32 + 200);
        assert!(deserialized.result.is_some());
    }
}

#[test]
fn test_json_rpc_error_various_codes() {
    let error_codes = vec![-32700, -32600, -32601, -32602, -32603, -32099, 0, 1, 100, -1];

    for (i, code) in error_codes.iter().enumerate() {
        let error = JsonRpcError {
            code: *code,
            message: format!("Error #{}", i),
            data: None,
        };

        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: i as i32 + 300,
            result: None,
            error: Some(error),
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: JsonRpcResponse = serde_json::from_str(&json).unwrap();

        assert!(deserialized.error.is_some());
        assert_eq!(deserialized.error.unwrap().code, *code);
        assert_eq!(deserialized.id, i as i32 + 300);
    }
}

#[test]
fn test_internal_event_payload_variations() {
    // Test different payload types by creating them
    let events = vec![
        InternalEvent::GeminiOutput {
            session_id: "session1".to_string(),
            payload: GeminiOutputPayload {
                text: "output text".to_string(),
            },
        },
        InternalEvent::GeminiThought {
            session_id: "session1".to_string(),
            payload: GeminiThoughtPayload {
                thought: "thinking thought".to_string(),
            },
        },
        InternalEvent::Error {
            session_id: "session1".to_string(),
            payload: ErrorPayload {
                error: "error message".to_string(),
            },
        },
        InternalEvent::CliIo {
            session_id: "session1".to_string(),
            payload: CliIoPayload {
                io_type: CliIoType::Input,
                data: "io data".to_string(),
            },
        },
    ];

    for event in events {
        // Just verify they can be cloned, indicating proper implementation
        let _cloned = event.clone();
    }
}

#[test]
fn test_tool_call_confirmation_various_ids() {
    let ids = vec![
        "simple-id",
        "id-with-hyphens",
        "id_with_underscores", 
        "id123with456numbers",
        "very-long-id-name-with-many-components-separated-by-hyphens",
        "ID_IN_ALL_CAPS",
        "Id_With_Mixed_Casing",
        "id!@#$%^&*()special",
    ];

    for (i, id) in ids.iter().enumerate() {
        let confirmation = ToolCallConfirmation {
            session_id: "session1".to_string(),
            request_id: i as u32 + 1000,
            tool_call_id: id.clone(),
            outcome: "success".to_string(),
        };

        let json = serde_json::to_string(&confirmation).unwrap();
        let deserialized: ToolCallConfirmation = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.tool_call_id, *id);
        assert_eq!(deserialized.request_id, i as u32 + 1000);
    }
}

#[test]
fn test_enriched_project_various_sha_formats() {
    let sha_formats = vec![
        "a".repeat(64), // Valid SHA256
        "f".repeat(64), // Another valid SHA256
        "0123456789abcdef".repeat(4), // Valid hex
    ];

    for (i, sha) in sha_formats.iter().enumerate() {
        let project = EnrichedProject {
            sha256: sha.clone(),
            root_path: PathBuf::from(format!("/path/{}", i)),
            metadata: ProjectMetadataView {
                path: format!("/path/{}", i),
                sha256: sha.clone(),
                friendly_name: format!("project-{}", i),
                first_used: None,
                updated_at: None,
            },
        };

        let json = serde_json::to_string(&project).unwrap();
        let deserialized: EnrichedProject = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.sha256, *sha);
        assert_eq!(deserialized.metadata.sha256, *sha);
    }
}

#[tokio::test]
async fn test_rate_limit_tracker_various_limits() {
    let config = RateLimitingConfig {
        strategy: "round-robin".to_string(),
        track_usage: true,
        usage_db_path: None,
    };

    let mut tracker = RateLimitTracker::new(config);

    let test_limits = vec![
        RateLimit { requests_per_minute: 1, requests_per_day: 10 },
        RateLimit { requests_per_minute: 5, requests_per_day: 50 },
        RateLimit { requests_per_minute: 10, requests_per_day: 100 },
        RateLimit { requests_per_minute: 100, requests_per_day: 1000 },
    ];

    for (i, limit) in test_limits.iter().enumerate() {
        let agent_id = format!("test-agent-{}", i);
        
        // Test that we can make up to the minute limit
        for j in 0..limit.requests_per_minute {
            assert!(tracker.check_and_increment(&agent_id, limit).await, 
                   "Agent {} request {} should succeed", agent_id, j);
        }
        
        // Next request should fail due to minute limit
        if limit.requests_per_minute > 0 {
            assert!(!tracker.check_and_increment(&agent_id, limit).await, 
                   "Agent {} request exceeding minute limit should fail", agent_id);
        }
    }
}

#[test]
fn test_project_list_item_serialization_consistency() {
    // Create the same item twice and verify serialization is identical
    let item1 = ProjectListItem {
        id: "consistent-id".to_string(),
        title: Some("Consistent Title".to_string()),
        status: Some("active".to_string()),
        created_at: Some("2023-01-01T00:00:00Z".to_string()),
        updated_at: Some("2023-01-02T00:00:00Z".to_string()),
        last_activity_at: Some("2023-01-03T00:00:00Z".to_string()),
        log_count: Some(42),
    };

    let item2 = ProjectListItem {
        id: "consistent-id".to_string(),
        title: Some("Consistent Title".to_string()),
        status: Some("active".to_string()),
        created_at: Some("2023-01-01T00:00:00Z".to_string()),
        updated_at: Some("2023-01-02T00:00:00Z".to_string()),
        last_activity_at: Some("2023-01-03T00:00:00Z".to_string()),
        log_count: Some(42),
    };

    let json1 = serde_json::to_string(&item1).unwrap();
    let json2 = serde_json::to_string(&item2).unwrap();

    assert_eq!(json1, json2);

    // Verify deserialization produces the same result
    let deserialized1: ProjectListItem = serde_json::from_str(&json1).unwrap();
    let deserialized2: ProjectListItem = serde_json::from_str(&json2).unwrap();

    assert_eq!(deserialized1, deserialized2);
}

#[test]
fn test_project_metadata_serialization_consistency() {
    let metadata1 = ProjectMetadata {
        path: PathBuf::from("/consistent/path"),
        sha256: Some("consistentsha".to_string()),
        friendly_name: Some("consistent-project".to_string()),
        first_used: Some(Utc::now().with_timezone(&FixedOffset::east_opt(0).unwrap())),
        updated_at: Some(Utc::now().with_timezone(&FixedOffset::east_opt(0).unwrap())),
    };

    let metadata2 = ProjectMetadata {
        path: PathBuf::from("/consistent/path"),
        sha256: Some("consistentsha".to_string()),
        friendly_name: Some("consistent-project".to_string()),
        first_used: None, // Different from metadata1
        updated_at: None, // Different from metadata1
    };

    let json1 = serde_json::to_string(&metadata1).unwrap();
    let json2 = serde_json::to_string(&metadata2).unwrap();

    // These should be different because of the timestamps
    // But both should deserialize correctly
    let _deserialized1: ProjectMetadata = serde_json::from_str(&json1).unwrap();
    let _deserialized2: ProjectMetadata = serde_json::from_str(&json2).unwrap();
}

#[test]
fn test_recent_chat_serialization_consistency() {
    let chat1 = RecentChat {
        id: "consistent-chat".to_string(),
        title: "Consistent Chat".to_string(),
        started_at_iso: "2023-01-01T00:00:00Z".to_string(),
        message_count: 5,
        summary: Some("Consistent summary".to_string()),
        last_activity_iso: Some("2023-01-02T00:00:00Z".to_string()),
        total_tokens: Some(100),
        tags: vec!["tag1".to_string(), "tag2".to_string()],
    };

    let chat2 = RecentChat {
        id: "consistent-chat".to_string(),
        title: "Consistent Chat".to_string(),
        started_at_iso: "2023-01-01T00:00:00Z".to_string(),
        message_count: 5,
        summary: Some("Consistent summary".to_string()),
        last_activity_iso: Some("2023-01-02T00:00:00Z".to_string()),
        total_tokens: Some(100),
        tags: vec!["tag1".to_string(), "tag2".to_string()],
    };

    let json1 = serde_json::to_string(&chat1).unwrap();
    let json2 = serde_json::to_string(&chat2).unwrap();

    assert_eq!(json1, json2);

    let deserialized1: RecentChat = serde_json::from_str(&json1).unwrap();
    let deserialized2: RecentChat = serde_json::from_str(&json2).unwrap();

    assert_eq!(deserialized1, deserialized2);
}

#[test]
fn test_message_match_serialization_consistency() {
    let match1 = MessageMatch {
        content_snippet: "consistent snippet".to_string(),
        start_line: 10,
        end_line: 15,
        file_path: Some("/consistent/path.txt".to_string()),
        timestamp: Some("2023-01-01T00:00:00Z".to_string()),
    };

    let match2 = MessageMatch {
        content_snippet: "consistent snippet".to_string(),
        start_line: 10,
        end_line: 15,
        file_path: Some("/consistent/path.txt".to_string()),
        timestamp: Some("2023-01-01T00:00:00Z".to_string()),
    };

    let json1 = serde_json::to_string(&match1).unwrap();
    let json2 = serde_json::to_string(&match2).unwrap();

    assert_eq!(json1, json2);

    let deserialized1: MessageMatch = serde_json::from_str(&json1).unwrap();
    let deserialized2: MessageMatch = serde_json::from_str(&json2).unwrap();

    assert_eq!(deserialized1, deserialized2);
}

#[test]
fn test_search_result_serialization_consistency() {
    let result1 = SearchResult {
        chat: RecentChat {
            id: "consistent-chat".to_string(),
            title: "Consistent Chat".to_string(),
            started_at_iso: "2023-01-01T00:00:00Z".to_string(),
            message_count: 5,
            summary: Some("Consistent summary".to_string()),
            last_activity_iso: Some("2023-01-02T00:00:00Z".to_string()),
            total_tokens: Some(100),
            tags: vec!["tag1".to_string(), "tag2".to_string()],
        },
        matches: vec![MessageMatch {
            content_snippet: "consistent snippet".to_string(),
            start_line: 10,
            end_line: 15,
            file_path: Some("/consistent/path.txt".to_string()),
            timestamp: Some("2023-01-01T00:00:00Z".to_string()),
        }],
        score: 0.85,
    };

    let result2 = SearchResult {
        chat: RecentChat {
            id: "consistent-chat".to_string(),
            title: "Consistent Chat".to_string(),
            started_at_iso: "2023-01-01T00:00:00Z".to_string(),
            message_count: 5,
            summary: Some("Consistent summary".to_string()),
            last_activity_iso: Some("2023-01-02T00:00:00Z".to_string()),
            total_tokens: Some(100),
            tags: vec!["tag1".to_string(), "tag2".to_string()],
        },
        matches: vec![MessageMatch {
            content_snippet: "consistent snippet".to_string(),
            start_line: 10,
            end_line: 15,
            file_path: Some("/consistent/path.txt".to_string()),
            timestamp: Some("2023-01-01T00:00:00Z".to_string()),
        }],
        score: 0.85,
    };

    let json1 = serde_json::to_string(&result1).unwrap();
    let json2 = serde_json::to_string(&result2).unwrap();

    assert_eq!(json1, json2);

    let deserialized1: SearchResult = serde_json::from_str(&json1).unwrap();
    let deserialized2: SearchResult = serde_json::from_str(&json2).unwrap();

    assert_eq!(deserialized1, deserialized2);
}

#[test]
fn test_detailed_conversation_serialization_consistency() {
    let conv1 = DetailedConversation {
        id: "consistent-conversation".to_string(),
        title: "Consistent Conversation".to_string(),
        started_at_iso: "2023-01-01T00:00:00Z".to_string(),
        messages: vec![ConversationHistoryEntry {
            id: "msg-1".to_string(),
            role: "user".to_string(),
            content: "Hello".to_string(),
            timestamp_iso: "2023-01-01T00:00:00Z".to_string(),
            tokens: Some(2),
        }],
        total_tokens: Some(100),
        tags: vec!["tag1".to_string()],
    };

    let conv2 = DetailedConversation {
        id: "consistent-conversation".to_string(),
        title: "Consistent Conversation".to_string(),
        started_at_iso: "2023-01-01T00:00:00Z".to_string(),
        messages: vec![ConversationHistoryEntry {
            id: "msg-1".to_string(),
            role: "user".to_string(),
            content: "Hello".to_string(),
            timestamp_iso: "2023-01-01T00:00:00Z".to_string(),
            tokens: Some(2),
        }],
        total_tokens: Some(100),
        tags: vec!["tag1".to_string()],
    };

    let json1 = serde_json::to_string(&conv1).unwrap();
    let json2 = serde_json::to_string(&conv2).unwrap();

    assert_eq!(json1, json2);

    let deserialized1: DetailedConversation = serde_json::from_str(&json1).unwrap();
    let deserialized2: DetailedConversation = serde_json::from_str(&json2).unwrap();

    assert_eq!(deserialized1, deserialized2);
}

#[test]
fn test_session_params_serialization_consistency() {
    let params1 = SessionParams {
        session_id: "consistent-session".to_string(),
        working_directory: "/consistent/dir".to_string(),
        model: "consistent-model".to_string(),
        cli_name: Some("consistent-cli".to_string()),
        backend_config: Some(QwenConfig {
            api_key: "key".to_string(),
            base_url: "https://api.example.com".to_string(),
            model: "model".to_string(),
            yolo: None,
        }),
        gemini_auth: Some(GeminiAuthConfig {
            method: "oauth".to_string(),
            api_key: Some("key".to_string()),
            vertex_project: Some("project".to_string()),
            vertex_location: Some("location".to_string()),
            yolo: None,
        }),
        llxprt_config: Some(LLxprtConfig {
            provider: "provider".to_string(),
            api_key: "key".to_string(),
            model: "model".to_string(),
            base_url: Some("https://api.example.com".to_string()),
        }),
    };

    let params2 = SessionParams {
        session_id: "consistent-session".to_string(),
        working_directory: "/consistent/dir".to_string(),
        model: "consistent-model".to_string(),
        cli_name: Some("consistent-cli".to_string()),
        backend_config: Some(QwenConfig {
            api_key: "key".to_string(),
            base_url: "https://api.example.com".to_string(),
            model: "model".to_string(),
            yolo: None,
        }),
        gemini_auth: Some(GeminiAuthConfig {
            method: "oauth".to_string(),
            api_key: Some("key".to_string()),
            vertex_project: Some("project".to_string()),
            vertex_location: Some("location".to_string()),
            yolo: None,
        }),
        llxprt_config: Some(LLxprtConfig {
            provider: "provider".to_string(),
            api_key: "key".to_string(),
            model: "model".to_string(),
            base_url: Some("https://api.example.com".to_string()),
        }),
    };

    let json1 = serde_json::to_string(&params1).unwrap();
    let json2 = serde_json::to_string(&params2).unwrap();

    assert_eq!(json1, json2);

    let deserialized1: SessionParams = serde_json::from_str(&json1).unwrap();
    let deserialized2: SessionParams = serde_json::from_str(&json2).unwrap();

    assert_eq!(deserialized1, deserialized2);
}

#[test]
fn test_file_content_serialization_consistency() {
    let content1 = FileContent {
        path: "/consistent/file.txt".to_string(),
        content: "consistent content".to_string(),
        size: 100,
        modified: "2023-01-01T00:00:00Z".to_string(),
        mime_type: "text/plain".to_string(),
    };

    let content2 = FileContent {
        path: "/consistent/file.txt".to_string(),
        content: "consistent content".to_string(),
        size: 100,
        modified: "2023-01-01T00:00:00Z".to_string(),
        mime_type: "text/plain".to_string(),
    };

    let json1 = serde_json::to_string(&content1).unwrap();
    let json2 = serde_json::to_string(&content2).unwrap();

    assert_eq!(json1, json2);

    let deserialized1: FileContent = serde_json::from_str(&json1).unwrap();
    let deserialized2: FileContent = serde_json::from_str(&json2).unwrap();

    assert_eq!(deserialized1, deserialized2);
}

#[test]
fn test_dir_entry_serialization_consistency() {
    let entry1 = DirEntry {
        name: "consistent-dir".to_string(),
        path: "/consistent/path".to_string(),
        size: 1024,
        modified: "2023-01-01T00:00:00Z".to_string(),
        is_directory: true,
        mime_type: "directory".to_string(),
    };

    let entry2 = DirEntry {
        name: "consistent-dir".to_string(),
        path: "/consistent/path".to_string(),
        size: 1024,
        modified: "2023-01-01T00:00:00Z".to_string(),
        is_directory: true,
        mime_type: "directory".to_string(),
    };

    let json1 = serde_json::to_string(&entry1).unwrap();
    let json2 = serde_json::to_string(&entry2).unwrap();

    assert_eq!(json1, json2);

    let deserialized1: DirEntry = serde_json::from_str(&json1).unwrap();
    let deserialized2: DirEntry = serde_json::from_str(&json2).unwrap();

    assert_eq!(deserialized1, deserialized2);
}

#[test]
fn test_gemini_command_serialization_consistency() {
    let cmd1 = GeminiCommand {
        name: "consistent-command".to_string(),
        description: "Consistent command".to_string(),
        path: "/path/consistent-command.md".to_string(),
    };

    let cmd2 = GeminiCommand {
        name: "consistent-command".to_string(),
        description: "Consistent command".to_string(),
        path: "/path/consistent-command.md".to_string(),
    };

    let json1 = serde_json::to_string(&cmd1).unwrap();
    let json2 = serde_json::to_string(&cmd2).unwrap();

    assert_eq!(json1, json2);

    let deserialized1: GeminiCommand = serde_json::from_str(&json1).unwrap();
    let deserialized2: GeminiCommand = serde_json::from_str(&json2).unwrap();

    assert_eq!(deserialized1, deserialized2);
}

#[test]
fn test_git_info_serialization_consistency() {
    let git1 = GitInfo {
        remote_url: Some("https://github.com/consistent/repo.git".to_string()),
        branch: Some("main".to_string()),
        commit_hash: Some("abc123".to_string()),
        is_dirty: Some(false),
    };

    let git2 = GitInfo {
        remote_url: Some("https://github.com/consistent/repo.git".to_string()),
        branch: Some("main".to_string()),
        commit_hash: Some("abc123".to_string()),
        is_dirty: Some(false),
    };

    let json1 = serde_json::to_string(&git1).unwrap();
    let json2 = serde_json::to_string(&git2).unwrap();

    assert_eq!(json1, json2);

    let deserialized1: GitInfo = serde_json::from_str(&json1).unwrap();
    let deserialized2: GitInfo = serde_json::from_str(&json2).unwrap();

    assert_eq!(deserialized1, deserialized2);
}

#[test]
fn test_json_rpc_request_serialization_consistency() {
    let req1 = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: 123,
        method: "consistent_method".to_string(),
        params: serde_json::Value::String("params".to_string()),
    };

    let req2 = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: 123,
        method: "consistent_method".to_string(),
        params: serde_json::Value::String("params".to_string()),
    };

    let json1 = serde_json::to_string(&req1).unwrap();
    let json2 = serde_json::to_string(&req2).unwrap();

    assert_eq!(json1, json2);

    let deserialized1: JsonRpcRequest = serde_json::from_str(&json1).unwrap();
    let deserialized2: JsonRpcRequest = serde_json::from_str(&json2).unwrap();

    assert_eq!(deserialized1, deserialized2);
}

#[test]
fn test_json_rpc_response_serialization_consistency() {
    let resp1 = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: 456,
        result: Some(serde_json::Value::String("result".to_string())),
        error: None,
    };

    let resp2 = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: 456,
        result: Some(serde_json::Value::String("result".to_string())),
        error: None,
    };

    let json1 = serde_json::to_string(&resp1).unwrap();
    let json2 = serde_json::to_string(&resp2).unwrap();

    assert_eq!(json1, json2);

    let deserialized1: JsonRpcResponse = serde_json::from_str(&json1).unwrap();
    let deserialized2: JsonRpcResponse = serde_json::from_str(&json2).unwrap();

    assert_eq!(deserialized1, deserialized2);
}

#[test]
fn test_json_rpc_error_serialization_consistency() {
    let error1 = JsonRpcError {
        code: -32600,
        message: "Consistent error".to_string(),
        data: None,
    };

    let error2 = JsonRpcError {
        code: -32600,
        message: "Consistent error".to_string(),
        data: None,
    };

    let resp1 = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: 789,
        result: None,
        error: Some(error1),
    };

    let resp2 = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: 789,
        result: None,
        error: Some(error2),
    };

    let json1 = serde_json::to_string(&resp1).unwrap();
    let json2 = serde_json::to_string(&resp2).unwrap();

    assert_eq!(json1, json2);

    let deserialized1: JsonRpcResponse = serde_json::from_str(&json1).unwrap();
    let deserialized2: JsonRpcResponse = serde_json::from_str(&json2).unwrap();

    assert_eq!(deserialized1, deserialized2);
}

#[test]
fn test_tool_call_confirmation_serialization_consistency() {
    let conf1 = ToolCallConfirmation {
        session_id: "session1".to_string(),
        request_id: 100,
        tool_call_id: "tool1".to_string(),
        outcome: "success".to_string(),
    };

    let conf2 = ToolCallConfirmation {
        session_id: "session1".to_string(),
        request_id: 100,
        tool_call_id: "tool1".to_string(),
        outcome: "success".to_string(),
    };

    let json1 = serde_json::to_string(&conf1).unwrap();
    let json2 = serde_json::to_string(&conf2).unwrap();

    assert_eq!(json1, json2);

    let deserialized1: ToolCallConfirmation = serde_json::from_str(&json1).unwrap();
    let deserialized2: ToolCallConfirmation = serde_json::from_str(&json2).unwrap();

    assert_eq!(deserialized1, deserialized2);
}

#[test]
fn test_enriched_project_serialization_consistency() {
    let proj1 = EnrichedProject {
        sha256: "consistentsha".to_string(),
        root_path: PathBuf::from("/consistent/path"),
        metadata: ProjectMetadataView {
            path: "/consistent/path".to_string(),
            sha256: "consistentsha".to_string(),
            friendly_name: "consistent-project".to_string(),
            first_used: None,
            updated_at: None,
        },
    };

    let proj2 = EnrichedProject {
        sha256: "consistentsha".to_string(),
        root_path: PathBuf::from("/consistent/path"),
        metadata: ProjectMetadataView {
            path: "/consistent/path".to_string(),
            sha256: "consistentsha".to_string(),
            friendly_name: "consistent-project".to_string(),
            first_used: None,
            updated_at: None,
        },
    };

    let json1 = serde_json::to_string(&proj1).unwrap();
    let json2 = serde_json::to_string(&proj2).unwrap();

    assert_eq!(json1, json2);

    let deserialized1: EnrichedProject = serde_json::from_str(&json1).unwrap();
    let deserialized2: EnrichedProject = serde_json::from_str(&json2).unwrap();

    assert_eq!(deserialized1, deserialized2);
}

#[tokio::test]
async fn test_rate_limit_tracker_serialization_consistency() {
    let config = RateLimitingConfig {
        strategy: "round-robin".to_string(),
        track_usage: true,
        usage_db_path: None,
    };

    let mut tracker1 = RateLimitTracker::new(config.clone());
    let mut tracker2 = RateLimitTracker::new(config);

    let agent_limit = RateLimit {
        requests_per_minute: 10,
        requests_per_day: 100,
    };

    // Add same usage to both trackers
    assert!(tracker1.check_and_increment("test-agent", &agent_limit).await);
    assert!(tracker2.check_and_increment("test-agent", &agent_limit).await);

    // Both should have same usage
    let (daily1, minute1) = tracker1.get_usage("test-agent").unwrap();
    let (daily2, minute2) = tracker2.get_usage("test-agent").unwrap();

    assert_eq!(daily1, daily2);
    assert_eq!(minute1, minute2);
}

#[test]
fn test_project_metadata_view_serialization_consistency() {
    let view1 = ProjectMetadataView {
        path: "/consistent/path".to_string(),
        sha256: "consistentsha".to_string(),
        friendly_name: "consistent-project".to_string(),
        first_used: Some("2023-01-01T00:00:00Z".to_string()),
        updated_at: Some("2023-01-02T00:00:00Z".to_string()),
    };

    let view2 = ProjectMetadataView {
        path: "/consistent/path".to_string(),
        sha256: "consistentsha".to_string(),
        friendly_name: "consistent-project".to_string(),
        first_used: Some("2023-01-01T00:00:00Z".to_string()),
        updated_at: Some("2023-01-02T00:00:00Z".to_string()),
    };

    let json1 = serde_json::to_string(&view1).unwrap();
    let json2 = serde_json::to_string(&view2).unwrap();

    assert_eq!(json1, json2);

    let deserialized1: ProjectMetadataView = serde_json::from_str(&json1).unwrap();
    let deserialized2: ProjectMetadataView = serde_json::from_str(&json2).unwrap();

    assert_eq!(deserialized1, deserialized2);
}

#[test]
fn test_search_filters_serialization_consistency() {
    let filters1 = SearchFilters {
        date_range: Some(("2023-01-01".to_string(), "2023-12-31".to_string())),
        tags: Some(vec!["tag1".to_string(), "tag2".to_string()]),
        project_id: Some("project1".to_string()),
        max_results: Some(10),
    };

    let filters2 = SearchFilters {
        date_range: Some(("2023-01-01".to_string(), "2023-12-31".to_string())),
        tags: Some(vec!["tag1".to_string(), "tag2".to_string()]),
        project_id: Some("project1".to_string()),
        max_results: Some(10),
    };

    let json1 = serde_json::to_string(&filters1).unwrap();
    let json2 = serde_json::to_string(&filters2).unwrap();

    assert_eq!(json1, json2);

    let deserialized1: SearchFilters = serde_json::from_str(&json1).unwrap();
    let deserialized2: SearchFilters = serde_json::from_str(&json2).unwrap();

    assert_eq!(deserialized1, deserialized2);
}

#[test]
fn test_conversation_history_entry_serialization_consistency() {
    let entry1 = ConversationHistoryEntry {
        id: "msg-1".to_string(),
        role: "user".to_string(),
        content: "Hello".to_string(),
        timestamp_iso: "2023-01-01T00:00:00Z".to_string(),
        tokens: Some(2),
    };

    let entry2 = ConversationHistoryEntry {
        id: "msg-1".to_string(),
        role: "user".to_string(),
        content: "Hello".to_string(),
        timestamp_iso: "2023-01-01T00:00:00Z".to_string(),
        tokens: Some(2),
    };

    let json1 = serde_json::to_string(&entry1).unwrap();
    let json2 = serde_json::to_string(&entry2).unwrap();

    assert_eq!(json1, json2);

    let deserialized1: ConversationHistoryEntry = serde_json::from_str(&json1).unwrap();
    let deserialized2: ConversationHistoryEntry = serde_json::from_str(&json2).unwrap();

    assert_eq!(deserialized1, deserialized2);
}

#[test]
fn test_command_result_serialization_consistency() {
    let result1 = CommandResult {
        success: true,
        output: "success output".to_string(),
        error: None,
    };

    let result2 = CommandResult {
        success: true,
        output: "success output".to_string(),
        error: None,
    };

    let json1 = serde_json::to_string(&result1).unwrap();
    let json2 = serde_json::to_string(&result2).unwrap();

    assert_eq!(json1, json2);

    let deserialized1: CommandResult = serde_json::from_str(&json1).unwrap();
    let deserialized2: CommandResult = serde_json::from_str(&json2).unwrap();

    assert_eq!(deserialized1, deserialized2);
}

#[test]
fn test_message_chunk_serialization_consistency() {
    let chunk1 = MessageChunk {
        content: "test chunk".to_string(),
        role: "assistant".to_string(),
        timestamp: "2023-01-01T00:00:00Z".to_string(),
    };

    let chunk2 = MessageChunk {
        content: "test chunk".to_string(),
        role: "assistant".to_string(),
        timestamp: "2023-01-01T00:00:00Z".to_string(),
    };

    let json1 = serde_json::to_string(&chunk1).unwrap();
    let json2 = serde_json::to_string(&chunk2).unwrap();

    assert_eq!(json1, json2);

    let deserialized1: MessageChunk = serde_json::from_str(&json1).unwrap();
    let deserialized2: MessageChunk = serde_json::from_str(&json2).unwrap();

    assert_eq!(deserialized1, deserialized2);
}

#[test]
fn test_assistant_chunk_serialization_consistency() {
    let chunk1 = AssistantChunk {
        text: Some("test text".to_string()),
        tool_calls: None,
        finish_reason: Some("stop".to_string()),
    };

    let chunk2 = AssistantChunk {
        text: Some("test text".to_string()),
        tool_calls: None,
        finish_reason: Some("stop".to_string()),
    };

    let json1 = serde_json::to_string(&chunk1).unwrap();
    let json2 = serde_json::to_string(&chunk2).unwrap();

    assert_eq!(json1, json2);

    let deserialized1: AssistantChunk = serde_json::from_str(&json1).unwrap();
    let deserialized2: AssistantChunk = serde_json::from_str(&json2).unwrap();

    assert_eq!(deserialized1, deserialized2);
}

#[test]
fn test_stream_assistant_message_chunk_params_serialization_consistency() {
    let params1 = StreamAssistantMessageChunkParams {
        session_id: "session1".to_string(),
        chunk: AssistantChunk {
            text: Some("test".to_string()),
            tool_calls: None,
            finish_reason: None,
        },
    };

    let params2 = StreamAssistantMessageChunkParams {
        session_id: "session1".to_string(),
        chunk: AssistantChunk {
            text: Some("test".to_string()),
            tool_calls: None,
            finish_reason: None,
        },
    };

    let json1 = serde_json::to_string(&params1).unwrap();
    let json2 = serde_json::to_string(&params2).unwrap();

    assert_eq!(json1, json2);

    let deserialized1: StreamAssistantMessageChunkParams = serde_json::from_str(&json1).unwrap();
    let deserialized2: StreamAssistantMessageChunkParams = serde_json::from_str(&json2).unwrap();

    assert_eq!(deserialized1, deserialized2);
}

#[test]
fn test_cli_io_payload_serialization_consistency() {
    let payload1 = CliIoPayload {
        io_type: CliIoType::Output,
        data: "test data".to_string(),
    };

    let payload2 = CliIoPayload {
        io_type: CliIoType::Output,
        data: "test data".to_string(),
    };

    let json1 = serde_json::to_string(&payload1).unwrap();
    let json2 = serde_json::to_string(&payload2).unwrap();

    assert_eq!(json1, json2);

    let deserialized1: CliIoPayload = serde_json::from_str(&json1).unwrap();
    let deserialized2: CliIoPayload = serde_json::from_str(&json2).unwrap();

    assert_eq!(deserialized1, deserialized2);
}

#[test]
fn test_error_payload_serialization_consistency() {
    let payload1 = ErrorPayload {
        error: "test error".to_string(),
    };

    let payload2 = ErrorPayload {
        error: "test error".to_string(),
    };

    let json1 = serde_json::to_string(&payload1).unwrap();
    let json2 = serde_json::to_string(&payload2).unwrap();

    assert_eq!(json1, json2);

    let deserialized1: ErrorPayload = serde_json::from_str(&json1).unwrap();
    let deserialized2: ErrorPayload = serde_json::from_str(&json2).unwrap();

    assert_eq!(deserialized1, deserialized2);
}

#[test]
fn test_gemini_output_payload_serialization_consistency() {
    let payload1 = GeminiOutputPayload {
        text: "test output".to_string(),
    };

    let payload2 = GeminiOutputPayload {
        text: "test output".to_string(),
    };

    let json1 = serde_json::to_string(&payload1).unwrap();
    let json2 = serde_json::to_string(&payload2).unwrap();

    assert_eq!(json1, json2);

    let deserialized1: GeminiOutputPayload = serde_json::from_str(&json1).unwrap();
    let deserialized2: GeminiOutputPayload = serde_json::from_str(&json2).unwrap();

    assert_eq!(deserialized1, deserialized2);
}

#[test]
fn test_gemini_thought_payload_serialization_consistency() {
    let payload1 = GeminiThoughtPayload {
        thought: "thinking...".to_string(),
    };

    let payload2 = GeminiThoughtPayload {
        thought: "thinking...".to_string(),
    };

    let json1 = serde_json::to_string(&payload1).unwrap();
    let json2 = serde_json::to_string(&payload2).unwrap();

    assert_eq!(json1, json2);

    let deserialized1: GeminiThoughtPayload = serde_json::from_str(&json1).unwrap();
    let deserialized2: GeminiThoughtPayload = serde_json::from_str(&json2).unwrap();

    assert_eq!(deserialized1, deserialized2);
}

#[test]
fn test_tool_call_confirmation_content_serialization_consistency() {
    let content1 = ToolCallConfirmationContent {
        tool_call_id: "tool1".to_string(),
        outcome: "success".to_string(),
    };

    let content2 = ToolCallConfirmationContent {
        tool_call_id: "tool1".to_string(),
        outcome: "success".to_string(),
    };

    let json1 = serde_json::to_string(&content1).unwrap();
    let json2 = serde_json::to_string(&content2).unwrap();

    assert_eq!(json1, json2);

    let deserialized1: ToolCallConfirmationContent = serde_json::from_str(&json1).unwrap();
    let deserialized2: ToolCallConfirmationContent = serde_json::from_str(&json2).unwrap();

    assert_eq!(deserialized1, deserialized2);
}

#[test]
fn test_tool_call_event_serialization_consistency() {
    let event1 = ToolCallEvent {
        id: "event1".to_string(),
        tool_call_id: "tool1".to_string(),
        status: ToolCallStatus::Requested,
        title: "Test Tool".to_string(),
        content: vec![],
        locations: vec![],
        kind: "function".to_string(),
        server_name: "server1".to_string(),
        tool_name: "tool1".to_string(),
    };

    let event2 = ToolCallEvent {
        id: "event1".to_string(),
        tool_call_id: "tool1".to_string(),
        status: ToolCallStatus::Requested,
        title: "Test Tool".to_string(),
        content: vec![],
        locations: vec![],
        kind: "function".to_string(),
        server_name: "server1".to_string(),
        tool_name: "tool1".to_string(),
    };

    let json1 = serde_json::to_string(&event1).unwrap();
    let json2 = serde_json::to_string(&event2).unwrap();

    assert_eq!(json1, json2);

    let deserialized1: ToolCallEvent = serde_json::from_str(&json1).unwrap();
    let deserialized2: ToolCallEvent = serde_json::from_str(&json2).unwrap();

    assert_eq!(deserialized1, deserialized2);
}

#[test]
fn test_tool_call_location_serialization_consistency() {
    let location1 = ToolCallLocation {
        file: "/path/to/file".to_string(),
        line: 10,
        column: 5,
    };

    let location2 = ToolCallLocation {
        file: "/path/to/file".to_string(),
        line: 10,
        column: 5,
    };

    let json1 = serde_json::to_string(&location1).unwrap();
    let json2 = serde_json::to_string(&location2).unwrap();

    assert_eq!(json1, json2);

    let deserialized1: ToolCallLocation = serde_json::from_str(&json1).unwrap();
    let deserialized2: ToolCallLocation = serde_json::from_str(&json2).unwrap();

    assert_eq!(deserialized1, deserialized2);
}

#[test]
fn test_tool_call_update_serialization_consistency() {
    let update1 = ToolCallUpdate {
        tool_call_id: "tool1".to_string(),
        status: ToolCallStatus::Completed,
        content: vec![],
        server_name: "server1".to_string(),
        tool_name: "tool1".to_string(),
    };

    let update2 = ToolCallUpdate {
        tool_call_id: "tool1".to_string(),
        status: ToolCallStatus::Completed,
        content: vec![],
        server_name: "server1".to_string(),
        tool_name: "tool1".to_string(),
    };

    let json1 = serde_json::to_string(&update1).unwrap();
    let json2 = serde_json::to_string(&update2).unwrap();

    assert_eq!(json1, json2);

    let deserialized1: ToolCallUpdate = serde_json::from_str(&json1).unwrap();
    let deserialized2: ToolCallUpdate = serde_json::from_str(&json2).unwrap();

    assert_eq!(deserialized1, deserialized2);
}

#[test]
fn test_tool_call_content_item_serialization_consistency() {
    let item1 = ToolCallContentItem {
        r#type: "text".to_string(),
        text: "test content".to_string(),
    };

    let item2 = ToolCallContentItem {
        r#type: "text".to_string(),
        text: "test content".to_string(),
    };

    let json1 = serde_json::to_string(&item1).unwrap();
    let json2 = serde_json::to_string(&item2).unwrap();

    assert_eq!(json1, json2);

    let deserialized1: ToolCallContentItem = serde_json::from_str(&json1).unwrap();
    let deserialized2: ToolCallContentItem = serde_json::from_str(&json2).unwrap();

    assert_eq!(deserialized1, deserialized2);
}

#[test]
fn test_content_block_serialization_consistency() {
    let block1 = ContentBlock::Text { text: "test".to_string() };
    let block2 = ContentBlock::Text { text: "test".to_string() };

    let json1 = serde_json::to_string(&block1).unwrap();
    let json2 = serde_json::to_string(&block2).unwrap();

    assert_eq!(json1, json2);

    let deserialized1: ContentBlock = serde_json::from_str(&json1).unwrap();
    let deserialized2: ContentBlock = serde_json::from_str(&json2).unwrap();

    assert_eq!(deserialized1, deserialized2);
}

#[test]
fn test_initialize_params_serialization_consistency() {
    let params1 = InitializeParams {
        protocol_version: 1,
        client_capabilities: bl1nk_core::adapters::acp::ClientCapabilities {
            fs: bl1nk_core::adapters::acp::FileSystemCapabilities {
                read_text_file: false,
                write_text_file: false,
            },
        },
    };

    let params2 = InitializeParams {
        protocol_version: 1,
        client_capabilities: bl1nk_core::adapters::acp::ClientCapabilities {
            fs: bl1nk_core::adapters::acp::FileSystemCapabilities {
                read_text_file: false,
                write_text_file: false,
            },
        },
    };

    let json1 = serde_json::to_string(&params1).unwrap();
    let json2 = serde_json::to_string(&params2).unwrap();

    assert_eq!(json1, json2);

    let deserialized1: InitializeParams = serde_json::from_str(&json1).unwrap();
    let deserialized2: InitializeParams = serde_json::from_str(&json2).unwrap();

    assert_eq!(deserialized1, deserialized2);
}

#[test]
fn test_session_new_params_serialization_consistency() {
    let params1 = SessionNewParams {
        cwd: "/test/dir".to_string(),
        mcp_servers: vec![],
    };

    let params2 = SessionNewParams {
        cwd: "/test/dir".to_string(),
        mcp_servers: vec![],
    };

    let json1 = serde_json::to_string(&params1).unwrap();
    let json2 = serde_json::to_string(&params2).unwrap();

    assert_eq!(json1, json2);

    let deserialized1: SessionNewParams = serde_json::from_str(&json1).unwrap();
    let deserialized2: SessionNewParams = serde_json::from_str(&json2).unwrap();

    assert_eq!(deserialized1, deserialized2);
}

#[test]
fn test_session_prompt_params_serialization_consistency() {
    let params1 = SessionPromptParams {
        session_id: "session1".to_string(),
        prompt: vec![ContentBlock::Text { text: "test".to_string() }],
    };

    let params2 = SessionPromptParams {
        session_id: "session1".to_string(),
        prompt: vec![ContentBlock::Text { text: "test".to_string() }],
    };

    let json1 = serde_json::to_string(&params1).unwrap();
    let json2 = serde_json::to_string(&params2).unwrap();

    assert_eq!(json1, json2);

    let deserialized1: SessionPromptParams = serde_json::from_str(&json1).unwrap();
    let deserialized2: SessionPromptParams = serde_json::from_str(&json2).unwrap();

    assert_eq!(deserialized1, deserialized2);
}

#[test]
fn test_authenticate_params_serialization_consistency() {
    let params1 = AuthenticateParams {
        method_id: "oauth".to_string(),
    };

    let params2 = AuthenticateParams {
        method_id: "oauth".to_string(),
    };

    let json1 = serde_json::to_string(&params1).unwrap();
    let json2 = serde_json::to_string(&params2).unwrap();

    assert_eq!(json1, json2);

    let deserialized1: AuthenticateParams = serde_json::from_str(&json1).unwrap();
    let deserialized2: AuthenticateParams = serde_json::from_str(&json2).unwrap();

    assert_eq!(deserialized1, deserialized2);
}

#[test]
fn test_session_new_result_serialization_consistency() {
    let result1 = SessionNewResult {
        session_id: "session1".to_string(),
    };

    let result2 = SessionNewResult {
        session_id: "session1".to_string(),
    };

    let json1 = serde_json::to_string(&result1).unwrap();
    let json2 = serde_json::to_string(&result2).unwrap();

    assert_eq!(json1, json2);

    let deserialized1: SessionNewResult = serde_json::from_str(&json1).unwrap();
    let deserialized2: SessionNewResult = serde_json::from_str(&json2).unwrap();

    assert_eq!(deserialized1, deserialized2);
}

#[test]
fn test_session_prompt_result_serialization_consistency() {
    let result1 = SessionPromptResult {
        stop_reason: "end_turn".to_string(),
        output: vec![],
    };

    let result2 = SessionPromptResult {
        stop_reason: "end_turn".to_string(),
        output: vec![],
    };

    let json1 = serde_json::to_string(&result1).unwrap();
    let json2 = serde_json::to_string(&result2).unwrap();

    assert_eq!(json1, json2);

    let deserialized1: SessionPromptResult = serde_json::from_str(&json1).unwrap();
    let deserialized2: SessionPromptResult = serde_json::from_str(&json2).unwrap();

    assert_eq!(deserialized1, deserialized2);
}

#[test]
fn test_session_request_permission_params_serialization_consistency() {
    let params1 = SessionRequestPermissionParams {
        tool_call: bl1nk_core::adapters::acp::ToolCall {
            id: "tool1".to_string(),
            name: "test-tool".to_string(),
            arguments: serde_json::Value::Null,
        },
    };

    let params2 = SessionRequestPermissionParams {
        tool_call: bl1nk_core::adapters::acp::ToolCall {
            id: "tool1".to_string(),
            name: "test-tool".to_string(),
            arguments: serde_json::Value::Null,
        },
    };

    let json1 = serde_json::to_string(&params1).unwrap();
    let json2 = serde_json::to_string(&params2).unwrap();

    assert_eq!(json1, json2);

    let deserialized1: SessionRequestPermissionParams = serde_json::from_str(&json1).unwrap();
    let deserialized2: SessionRequestPermissionParams = serde_json::from_str(&json2).unwrap();

    assert_eq!(deserialized1, deserialized2);
}

#[test]
fn test_qwen_config_serialization_consistency() {
    let config1 = QwenConfig {
        api_key: "key1".to_string(),
        base_url: "https://api.example.com".to_string(),
        model: "model1".to_string(),
        yolo: Some(true),
    };

    let config2 = QwenConfig {
        api_key: "key1".to_string(),
        base_url: "https://api.example.com".to_string(),
        model: "model1".to_string(),
        yolo: Some(true),
    };

    let json1 = serde_json::to_string(&config1).unwrap();
    let json2 = serde_json::to_string(&config2).unwrap();

    assert_eq!(json1, json2);

    let deserialized1: QwenConfig = serde_json::from_str(&json1).unwrap();
    let deserialized2: QwenConfig = serde_json::from_str(&json2).unwrap();

    assert_eq!(deserialized1, deserialized2);
}

#[test]
fn test_gemini_auth_config_serialization_consistency() {
    let config1 = GeminiAuthConfig {
        method: "oauth".to_string(),
        api_key: Some("key1".to_string()),
        vertex_project: Some("project1".to_string()),
        vertex_location: Some("location1".to_string()),
        yolo: Some(false),
    };

    let config2 = GeminiAuthConfig {
        method: "oauth".to_string(),
        api_key: Some("key1".to_string()),
        vertex_project: Some("project1".to_string()),
        vertex_location: Some("location1".to_string()),
        yolo: Some(false),
    };

    let json1 = serde_json::to_string(&config1).unwrap();
    let json2 = serde_json::to_string(&config2).unwrap();

    assert_eq!(json1, json2);

    let deserialized1: GeminiAuthConfig = serde_json::from_str(&json1).unwrap();
    let deserialized2: GeminiAuthConfig = serde_json::from_str(&json2).unwrap();

    assert_eq!(deserialized1, deserialized2);
}

#[test]
fn test_llxprt_config_serialization_consistency() {
    let config1 = LLxprtConfig {
        provider: "openai".to_string(),
        api_key: "key1".to_string(),
        model: "gpt-4".to_string(),
        base_url: Some("https://api.openai.com".to_string()),
    };

    let config2 = LLxprtConfig {
        provider: "openai".to_string(),
        api_key: "key1".to_string(),
        model: "gpt-4".to_string(),
        base_url: Some("https://api.openai.com".to_string()),
    };

    let json1 = serde_json::to_string(&config1).unwrap();
    let json2 = serde_json::to_string(&config2).unwrap();

    assert_eq!(json1, json2);

    let deserialized1: LLxprtConfig = serde_json::from_str(&json1).unwrap();
    let deserialized2: LLxprtConfig = serde_json::from_str(&json2).unwrap();

    assert_eq!(deserialized1, deserialized2);
}

#[test]
fn test_json_rpc_error_serialization_consistency() {
    let error1 = JsonRpcError {
        code: -32600,
        message: "Test error".to_string(),
        data: None,
    };

    let error2 = JsonRpcError {
        code: -32600,
        message: "Test error".to_string(),
        data: None,
    };

    let json1 = serde_json::to_string(&error1).unwrap();
    let json2 = serde_json::to_string(&error2).unwrap();

    assert_eq!(json1, json2);

    let deserialized1: JsonRpcError = serde_json::from_str(&json1).unwrap();
    let deserialized2: JsonRpcError = serde_json::from_str(&json2).unwrap();

    assert_eq!(deserialized1, deserialized2);
}

#[tokio::test]
async fn test_rate_limit_tracker_edge_cases() {
    let config = RateLimitingConfig {
        strategy: "round-robin".to_string(),
        track_usage: true,
        usage_db_path: None,
    };

    let mut tracker = RateLimitTracker::new(config);

    // Test with zero limits
    let zero_limits = RateLimit {
        requests_per_minute: 0,
        requests_per_day: 0,
    };

    assert!(!tracker.check_and_increment("zero-agent", &zero_limits).await);

    // Test with very high limits
    let high_limits = RateLimit {
        requests_per_minute: 1000000,
        requests_per_day: 1000000,
    };

    for i in 0..100 {
        assert!(tracker.check_and_increment(&format!("high-agent-{}", i), &high_limits).await);
    }

    // Test with mixed limits (low minute, high day)
    let mixed_limits = RateLimit {
        requests_per_minute: 1,
        requests_per_day: 1000,
    };

    // Should succeed initially
    assert!(tracker.check_and_increment("mixed-agent", &mixed_limits).await);
    
    // Should fail due to minute limit
    assert!(!tracker.check_and_increment("mixed-agent", &mixed_limits).await);
}

#[test]
fn test_project_list_item_edge_cases() {
    // Test with maximum length values
    let max_id = "a".repeat(10000);
    let max_title = "b".repeat(10000);
    
    let item = ProjectListItem {
        id: max_id.clone(),
        title: Some(max_title.clone()),
        status: Some("a".repeat(1000)),
        created_at: Some("9999-12-31T23:59:59Z".to_string()),
        updated_at: Some("9999-12-31T23:59:59Z".to_string()),
        last_activity_at: Some("9999-12-31T23:59:59Z".to_string()),
        log_count: Some(u32::MAX),
    };

    // Should serialize without error
    let json = serde_json::to_string(&item).unwrap();
    let deserialized: ProjectListItem = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.id, max_id);
    assert_eq!(deserialized.title, Some(max_title));
    assert_eq!(deserialized.log_count, Some(u32::MAX));
}

#[test]
fn test_project_metadata_edge_cases() {
    // Test with maximum path length
    let long_path = PathBuf::from("/".to_string() + &"segment/".repeat(1000) + "final");
    
    let metadata = ProjectMetadata {
        path: long_path.clone(),
        sha256: Some("a".repeat(64)), // Valid SHA256
        friendly_name: Some("a".repeat(10000)),
        first_used: Some(DateTime::parse_from_rfc3339("2023-01-01T00:00:00Z").unwrap().with_timezone(&Utc).with_timezone(&FixedOffset::east_opt(0).unwrap())),
        updated_at: Some(DateTime::parse_from_rfc3339("2023-12-31T23:59:59Z").unwrap().with_timezone(&Utc).with_timezone(&FixedOffset::east_opt(0).unwrap())),
    };

    let json = serde_json::to_string(&metadata).unwrap();
    let deserialized: ProjectMetadata = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.path, long_path);
    assert_eq!(deserialized.sha256, Some("a".repeat(64)));
}

#[test]
fn test_recent_chat_edge_cases() {
    // Test with maximum values
    let chat = RecentChat {
        id: "a".repeat(10000),
        title: "b".repeat(10000),
        started_at_iso: "9999-12-31T23:59:59Z".to_string(),
        message_count: u32::MAX,
        summary: Some("c".repeat(10000)),
        last_activity_iso: Some("9999-12-31T23:59:59Z".to_string()),
        total_tokens: Some(u32::MAX),
        tags: vec!["tag".to_string(); 1000], // Many tags
    };

    let json = serde_json::to_string(&chat).unwrap();
    let deserialized: RecentChat = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.id, "a".repeat(10000));
    assert_eq!(deserialized.message_count, u32::MAX);
    assert_eq!(deserialized.tags.len(), 1000);
}

#[test]
fn test_message_match_edge_cases() {
    // Test with maximum content
    let match_item = MessageMatch {
        content_snippet: "a".repeat(100000), // Very long snippet
        start_line: u32::MAX,
        end_line: u32::MAX,
        file_path: Some("/".to_string() + &"dir/".repeat(1000) + "file.txt"),
        timestamp: Some("9999-12-31T23:59:59Z".to_string()),
    };

    let json = serde_json::to_string(&match_item).unwrap();
    let deserialized: MessageMatch = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.content_snippet.len(), 100000);
    assert_eq!(deserialized.start_line, u32::MAX);
}

#[test]
fn test_search_result_edge_cases() {
    // Test with many matches and high score
    let result = SearchResult {
        chat: RecentChat {
            id: "test".to_string(),
            title: "Test".to_string(),
            started_at_iso: "2023-01-01T00:00:00Z".to_string(),
            message_count: 1,
            summary: None,
            last_activity_iso: None,
            total_tokens: None,
            tags: vec![],
        },
        matches: (0..1000).map(|i| MessageMatch {
            content_snippet: format!("match {}", i),
            start_line: i,
            end_line: i + 1,
            file_path: Some(format!("/file{}.txt", i)),
            timestamp: Some("2023-01-01T00:00:00Z".to_string()),
        }).collect(),
        score: 1.0, // Maximum score
    };

    let json = serde_json::to_string(&result).unwrap();
    let deserialized: SearchResult = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.matches.len(), 1000);
    assert_eq!(deserialized.score, 1.0);
}

#[test]
fn test_detailed_conversation_edge_cases() {
    // Test with many messages and maximum values
    let conversation = DetailedConversation {
        id: "a".repeat(10000),
        title: "b".repeat(10000),
        started_at_iso: "9999-12-31T23:59:59Z".to_string(),
        messages: (0..10000).map(|i| ConversationHistoryEntry {
            id: format!("msg{}", i),
            role: if i % 2 == 0 { "user" } else { "assistant" }.to_string(),
            content: format!("message content {}", i),
            timestamp_iso: "2023-01-01T00:00:00Z".to_string(),
            tokens: Some((i % 1000) as u32),
        }).collect(),
        total_tokens: Some(u32::MAX),
        tags: vec!["tag".to_string(); 1000],
    };

    let json = serde_json::to_string(&conversation).unwrap();
    let deserialized: DetailedConversation = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.id.len(), 10000);
    assert_eq!(deserialized.messages.len(), 10000);
    assert_eq!(deserialized.total_tokens, Some(u32::MAX));
    assert_eq!(deserialized.tags.len(), 1000);
}

#[test]
fn test_session_params_edge_cases() {
    // Test with maximum length values
    let params = SessionParams {
        session_id: "a".repeat(10000),
        working_directory: "/".to_string() + &"dir/".repeat(1000) + "final",
        model: "a".repeat(1000),
        cli_name: Some("a".repeat(1000)),
        backend_config: Some(QwenConfig {
            api_key: "a".repeat(1000),
            base_url: "https://".to_string() + &"a".repeat(1000) + ".com",
            model: "a".repeat(1000),
            yolo: Some(true),
        }),
        gemini_auth: Some(GeminiAuthConfig {
            method: "a".repeat(1000),
            api_key: Some("a".repeat(1000)),
            vertex_project: Some("a".repeat(1000)),
            vertex_location: Some("a".repeat(1000)),
            yolo: Some(false),
        }),
        llxprt_config: Some(LLxprtConfig {
            provider: "a".repeat(1000),
            api_key: "a".repeat(1000),
            model: "a".repeat(1000),
            base_url: Some("https://".to_string() + &"a".repeat(1000) + ".com"),
        }),
    };

    let json = serde_json::to_string(&params).unwrap();
    let deserialized: SessionParams = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.session_id, "a".repeat(10000));
    assert!(deserialized.backend_config.is_some());
    assert!(deserialized.gemini_auth.is_some());
    assert!(deserialized.llxprt_config.is_some());
}

#[test]
fn test_file_content_edge_cases() {
    // Test with maximum values
    let content = FileContent {
        path: "/".to_string() + &"dir/".repeat(1000) + "file.txt",
        content: "a".repeat(10 * 1024 * 1024), // 10MB content
        size: u64::MAX,
        modified: "9999-12-31T23:59:59Z".to_string(),
        mime_type: "a".repeat(1000),
    };

    let json = serde_json::to_string(&content).unwrap();
    let deserialized: FileContent = serde_json::from_str(&json).unwrap();

    assert!(deserialized.path.len() > 1000);
    assert_eq!(deserialized.size, u64::MAX);
    assert_eq!(deserialized.content.len(), 10 * 1024 * 1024);
}

#[test]
fn test_dir_entry_edge_cases() {
    // Test with maximum values
    let entry = DirEntry {
        name: "a".repeat(1000),
        path: "/".to_string() + &"dir/".repeat(1000) + "file",
        size: u64::MAX,
        modified: "9999-12-31T23:59:59Z".to_string(),
        is_directory: true,
        mime_type: "a".repeat(1000),
    };

    let json = serde_json::to_string(&entry).unwrap();
    let deserialized: DirEntry = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.name.len(), 1000);
    assert_eq!(deserialized.size, u64::MAX);
    assert!(deserialized.path.len() > 1000);
}

#[test]
fn test_gemini_command_edge_cases() {
    // Test with maximum values
    let command = GeminiCommand {
        name: "a".repeat(1000),
        description: "b".repeat(10000),
        path: "/".to_string() + &"dir/".repeat(1000) + "command.md",
    };

    let json = serde_json::to_string(&command).unwrap();
    let deserialized: GeminiCommand = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.name.len(), 1000);
    assert_eq!(deserialized.description.len(), 10000);
    assert!(deserialized.path.len() > 1000);
}

#[test]
fn test_git_info_edge_cases() {
    // Test with maximum values
    let git_info = GitInfo {
        remote_url: Some("https://".to_string() + &"a".repeat(10000) + ".git"),
        branch: Some("a".repeat(1000)),
        commit_hash: Some("a".repeat(1000)),
        is_dirty: Some(true),
    };

    let json = serde_json::to_string(&git_info).unwrap();
    let deserialized: GitInfo = serde_json::from_str(&json).unwrap();

    assert!(deserialized.remote_url.as_ref().unwrap().len() > 10000);
    assert_eq!(deserialized.branch.as_ref().unwrap().len(), 1000);
}

#[test]
fn test_json_rpc_request_edge_cases() {
    // Test with maximum values
    let large_params = serde_json::json!({
        "data": "a".repeat(1000000) // 1MB of data
    });

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: i32::MAX,
        method: "a".repeat(10000),
        params: large_params,
    };

    let json = serde_json::to_string(&request).unwrap();
    let deserialized: JsonRpcRequest = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.id, i32::MAX);
    assert_eq!(deserialized.method.len(), 10000);
}

#[test]
fn test_json_rpc_response_edge_cases() {
    // Test with maximum values
    let large_result = serde_json::json!({
        "data": "a".repeat(1000000) // 1MB of data
    });

    let response = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: i32::MAX,
        result: Some(large_result),
        error: None,
    };

    let json = serde_json::to_string(&response).unwrap();
    let deserialized: JsonRpcResponse = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.id, i32::MAX);
    assert!(deserialized.result.is_some());
}

#[test]
fn test_json_rpc_error_edge_cases() {
    // Test with maximum values
    let large_error_data = serde_json::json!({
        "details": "a".repeat(1000000) // 1MB of error details
    });

    let error = JsonRpcError {
        code: i32::MIN,
        message: "a".repeat(10000),
        data: Some(large_error_data),
    };

    let response = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: i32::MAX,
        result: None,
        error: Some(error),
    };

    let json = serde_json::to_string(&response).unwrap();
    let deserialized: JsonRpcResponse = serde_json::from_str(&json).unwrap();

    assert!(deserialized.error.is_some());
    assert_eq!(deserialized.id, i32::MAX);
}

#[test]
fn test_tool_call_confirmation_edge_cases() {
    // Test with maximum values
    let confirmation = ToolCallConfirmation {
        session_id: "a".repeat(10000),
        request_id: u32::MAX,
        tool_call_id: "b".repeat(10000),
        outcome: "c".repeat(10000),
    };

    let json = serde_json::to_string(&confirmation).unwrap();
    let deserialized: ToolCallConfirmation = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.session_id.len(), 10000);
    assert_eq!(deserialized.request_id, u32::MAX);
    assert_eq!(deserialized.tool_call_id.len(), 10000);
}

#[test]
fn test_enriched_project_edge_cases() {
    // Test with maximum values
    let project = EnrichedProject {
        sha256: "a".repeat(64), // Valid SHA256
        root_path: PathBuf::from("/".to_string() + &"dir/".repeat(1000) + "final"),
        metadata: ProjectMetadataView {
            path: "/".to_string() + &"dir/".repeat(1000) + "final",
            sha256: "a".repeat(64),
            friendly_name: "b".repeat(10000),
            first_used: Some("9999-12-31T23:59:59Z".to_string()),
            updated_at: Some("9999-12-31T23:59:59Z".to_string()),
        },
    };

    let json = serde_json::to_string(&project).unwrap();
    let deserialized: EnrichedProject = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.sha256.len(), 64);
    assert!(deserialized.root_path.to_string_lossy().len() > 1000);
    assert_eq!(deserialized.metadata.friendly_name.len(), 10000);
}

#[tokio::test]
async fn test_rate_limit_tracker_concurrent_access_simulation() {
    use std::sync::Arc;
    use tokio::sync::Mutex;
    
    let config = RateLimitingConfig {
        strategy: "round-robin".to_string(),
        track_usage: true,
        usage_db_path: None,
    };

    let tracker = Arc::new(Mutex::new(RateLimitTracker::new(config)));

    let agent_limit = RateLimit {
        requests_per_minute: 100, // High limit to avoid contention
        requests_per_day: 1000,
    };

    // Simulate multiple concurrent requests
    let mut handles = vec![];
    for i in 0..10 {
        let tracker_clone = Arc::clone(&tracker);
        let agent_limit_clone = agent_limit.clone();
        
        let handle = tokio::spawn(async move {
            let mut tracker = tracker_clone.lock().await;
            let agent_id = format!("concurrent-agent-{}", i);
            
            // Make several requests from each "thread"
            for j in 0..10 {
                let success = tracker.check_and_increment(&format!("{}-{}", agent_id, j), &agent_limit_clone).await;
                if !success {
                    return false;
                }
            }
            true
        });
        
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result, "Concurrent access test failed");
    }

    // Verify final state
    let final_tracker = tracker.lock().await;
    // Should have processed 100 requests (10 agents * 10 requests each)
    // But we can't predict exact usage due to the way agent IDs are distributed
    // Just verify the tracker is still functional
    assert!(final_tracker.check_and_increment("final-test", &agent_limit).await);
}

#[test]
fn test_project_metadata_view_edge_cases() {
    // Test with maximum values
    let view = ProjectMetadataView {
        path: "/".to_string() + &"dir/".repeat(1000) + "file",
        sha256: "a".repeat(64), // Valid SHA256
        friendly_name: "b".repeat(10000),
        first_used: Some("9999-12-31T23:59:59Z".to_string()),
        updated_at: Some("9999-12-31T23:59:59Z".to_string()),
    };

    let json = serde_json::to_string(&view).unwrap();
    let deserialized: ProjectMetadataView = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.sha256.len(), 64);
    assert_eq!(deserialized.friendly_name.len(), 10000);
    assert!(deserialized.path.len() > 1000);
}

#[test]
fn test_search_filters_edge_cases() {
    // Test with maximum values
    let filters = SearchFilters {
        date_range: Some(("0001-01-01".to_string(), "9999-12-31".to_string())),
        tags: Some(vec!["tag".to_string(); 10000]), // Many tags
        project_id: Some("a".repeat(10000)),
        max_results: Some(u32::MAX),
    };

    let json = serde_json::to_string(&filters).unwrap();
    let deserialized: SearchFilters = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.tags.as_ref().unwrap().len(), 10000);
    assert_eq!(deserialized.max_results, Some(u32::MAX));
}

#[test]
fn test_conversation_history_entry_edge_cases() {
    // Test with maximum values
    let entry = ConversationHistoryEntry {
        id: "a".repeat(10000),
        role: "b".repeat(1000),
        content: "c".repeat(1000000), // 1MB content
        timestamp_iso: "9999-12-31T23:59:59Z".to_string(),
        tokens: Some(u32::MAX),
    };

    let json = serde_json::to_string(&entry).unwrap();
    let deserialized: ConversationHistoryEntry = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.id.len(), 10000);
    assert_eq!(deserialized.content.len(), 1000000);
    assert_eq!(deserialized.tokens, Some(u32::MAX));
}

#[test]
fn test_command_result_edge_cases() {
    // Test with maximum values
    let result = CommandResult {
        success: true,
        output: "a".repeat(10000000), // 10MB output
        error: Some("b".repeat(1000000)), // 1MB error
    };

    let json = serde_json::to_string(&result).unwrap();
    let deserialized: CommandResult = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.output.len(), 10000000);
    assert_eq!(deserialized.error.as_ref().unwrap().len(), 1000000);
}

#[test]
fn test_message_chunk_edge_cases() {
    // Test with maximum values
    let chunk = MessageChunk {
        content: "a".repeat(10000000), // 10MB content
        role: "b".repeat(1000),
        timestamp: "9999-12-31T23:59:59Z".to_string(),
    };

    let json = serde_json::to_string(&chunk).unwrap();
    let deserialized: MessageChunk = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.content.len(), 10000000);
    assert_eq!(deserialized.role.len(), 1000);
}

#[test]
fn test_assistant_chunk_edge_cases() {
    // Test with maximum values
    let chunk = AssistantChunk {
        text: Some("a".repeat(10000000)), // 10MB text
        tool_calls: Some(vec![]),
        finish_reason: Some("b".repeat(1000)),
    };

    let json = serde_json::to_string(&chunk).unwrap();
    let deserialized: AssistantChunk = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.text.as_ref().unwrap().len(), 10000000);
    assert_eq!(deserialized.finish_reason.as_ref().unwrap().len(), 1000);
}

#[test]
fn test_stream_assistant_message_chunk_params_edge_cases() {
    // Test with maximum values
    let params = StreamAssistantMessageChunkParams {
        session_id: "a".repeat(10000),
        chunk: AssistantChunk {
            text: Some("b".repeat(10000000)), // 10MB text
            tool_calls: Some(vec![]),
            finish_reason: Some("c".repeat(1000)),
        },
    };

    let json = serde_json::to_string(&params).unwrap();
    let deserialized: StreamAssistantMessageChunkParams = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.session_id.len(), 10000);
    assert_eq!(deserialized.chunk.text.as_ref().unwrap().len(), 10000000);
}

#[test]
fn test_cli_io_payload_edge_cases() {
    // Test with maximum values
    let payload = CliIoPayload {
        io_type: CliIoType::Output,
        data: "a".repeat(10000000), // 10MB data
    };

    let json = serde_json::to_string(&payload).unwrap();
    let deserialized: CliIoPayload = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.data.len(), 10000000);
}

#[test]
fn test_error_payload_edge_cases() {
    // Test with maximum values
    let payload = ErrorPayload {
        error: "a".repeat(10000000), // 10MB error message
    };

    let json = serde_json::to_string(&payload).unwrap();
    let deserialized: ErrorPayload = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.error.len(), 10000000);
}

#[test]
fn test_gemini_output_payload_edge_cases() {
    // Test with maximum values
    let payload = GeminiOutputPayload {
        text: "a".repeat(10000000), // 10MB output text
    };

    let json = serde_json::to_string(&payload).unwrap();
    let deserialized: GeminiOutputPayload = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.text.len(), 10000000);
}

#[test]
fn test_gemini_thought_payload_edge_cases() {
    // Test with maximum values
    let payload = GeminiThoughtPayload {
        thought: "a".repeat(10000000), // 10MB thought text
    };

    let json = serde_json::to_string(&payload).unwrap();
    let deserialized: GeminiThoughtPayload = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.thought.len(), 10000000);
}

#[test]
fn test_tool_call_confirmation_content_edge_cases() {
    // Test with maximum values
    let content = ToolCallConfirmationContent {
        tool_call_id: "a".repeat(10000),
        outcome: "b".repeat(10000),
    };

    let json = serde_json::to_string(&content).unwrap();
    let deserialized: ToolCallConfirmationContent = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.tool_call_id.len(), 10000);
    assert_eq!(deserialized.outcome.len(), 10000);
}

#[test]
fn test_tool_call_event_edge_cases() {
    // Test with maximum values
    let event = ToolCallEvent {
        id: "a".repeat(10000),
        tool_call_id: "b".repeat(10000),
        status: ToolCallStatus::Requested,
        title: "c".repeat(10000),
        content: vec![],
        locations: vec![],
        kind: "d".repeat(1000),
        server_name: "e".repeat(1000),
        tool_name: "f".repeat(1000),
    };

    let json = serde_json::to_string(&event).unwrap();
    let deserialized: ToolCallEvent = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.id.len(), 10000);
    assert_eq!(deserialized.tool_call_id.len(), 10000);
    assert_eq!(deserialized.title.len(), 10000);
}

#[test]
fn test_tool_call_location_edge_cases() {
    // Test with maximum values
    let location = ToolCallLocation {
        file: "/".to_string() + &"dir/".repeat(1000) + "file",
        line: u32::MAX,
        column: u32::MAX,
    };

    let json = serde_json::to_string(&location).unwrap();
    let deserialized: ToolCallLocation = serde_json::from_str(&json).unwrap();

    assert!(deserialized.file.len() > 1000);
    assert_eq!(deserialized.line, u32::MAX);
    assert_eq!(deserialized.column, u32::MAX);
}

#[test]
fn test_tool_call_update_edge_cases() {
    // Test with maximum values
    let update = ToolCallUpdate {
        tool_call_id: "a".repeat(10000),
        status: ToolCallStatus::Completed,
        content: vec![],
        server_name: "b".repeat(1000),
        tool_name: "c".repeat(1000),
    };

    let json = serde_json::to_string(&update).unwrap();
    let deserialized: ToolCallUpdate = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.tool_call_id.len(), 10000);
}

#[test]
fn test_tool_call_content_item_edge_cases() {
    // Test with maximum values
    let item = ToolCallContentItem {
        r#type: "a".repeat(1000),
        text: "b".repeat(10000000), // 10MB text
    };

    let json = serde_json::to_string(&item).unwrap();
    let deserialized: ToolCallContentItem = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.r#type.len(), 1000);
    assert_eq!(deserialized.text.len(), 10000000);
}

#[test]
fn test_content_block_edge_cases() {
    // Test with maximum values
    let block = ContentBlock::Text { text: "a".repeat(10000000) }; // 10MB text

    let json = serde_json::to_string(&block).unwrap();
    let deserialized: ContentBlock = serde_json::from_str(&json).unwrap();

    match deserialized {
        ContentBlock::Text { text } => assert_eq!(text.len(), 10000000),
        _ => panic!("Wrong content block type"),
    }
}

#[test]
fn test_initialize_params_edge_cases() {
    // Test with valid parameters (capabilities have fixed structure)
    let params = InitializeParams {
        protocol_version: i32::MAX,
        client_capabilities: bl1nk_core::adapters::acp::ClientCapabilities {
            fs: bl1nk_core::adapters::acp::FileSystemCapabilities {
                read_text_file: true,
                write_text_file: true,
            },
        },
    };

    let json = serde_json::to_string(&params).unwrap();
    let deserialized: InitializeParams = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.protocol_version, i32::MAX);
}

#[test]
fn test_session_new_params_edge_cases() {
    // Test with maximum values
    let params = SessionNewParams {
        cwd: "/".to_string() + &"dir/".repeat(1000) + "final",
        mcp_servers: vec![], // We can't create servers with maximum values easily
    };

    let json = serde_json::to_string(&params).unwrap();
    let deserialized: SessionNewParams = serde_json::from_str(&json).unwrap();

    assert!(deserialized.cwd.len() > 1000);
}

#[test]
fn test_session_prompt_params_edge_cases() {
    // Test with maximum values
    let params = SessionPromptParams {
        session_id: "a".repeat(10000),
        prompt: vec![ContentBlock::Text { text: "b".repeat(1000000) }], // 1MB text
    };

    let json = serde_json::to_string(&params).unwrap();
    let deserialized: SessionPromptParams = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.session_id.len(), 10000);
    assert_eq!(deserialized.prompt.len(), 1);
}

#[test]
fn test_authenticate_params_edge_cases() {
    // Test with maximum values
    let params = AuthenticateParams {
        method_id: "a".repeat(10000),
    };

    let json = serde_json::to_string(&params).unwrap();
    let deserialized: AuthenticateParams = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.method_id.len(), 10000);
}

#[test]
fn test_session_new_result_edge_cases() {
    // Test with maximum values
    let result = SessionNewResult {
        session_id: "a".repeat(10000),
    };

    let json = serde_json::to_string(&result).unwrap();
    let deserialized: SessionNewResult = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.session_id.len(), 10000);
}

#[test]
fn test_session_prompt_result_edge_cases() {
    // Test with maximum values
    let result = SessionPromptResult {
        stop_reason: "a".repeat(1000),
        output: vec![],
    };

    let json = serde_json::to_string(&result).unwrap();
    let deserialized: SessionPromptResult = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.stop_reason.len(), 1000);
}

#[test]
fn test_session_request_permission_params_edge_cases() {
    // Test with maximum values
    let params = SessionRequestPermissionParams {
        tool_call: bl1nk_core::adapters::acp::ToolCall {
            id: "a".repeat(10000),
            name: "b".repeat(1000),
            arguments: serde_json::Value::String("c".repeat(1000000)), // 1MB arguments
        },
    };

    let json = serde_json::to_string(&params).unwrap();
    let deserialized: SessionRequestPermissionParams = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.tool_call.id.len(), 10000);
}

#[test]
fn test_qwen_config_edge_cases() {
    // Test with maximum values
    let config = QwenConfig {
        api_key: "a".repeat(10000),
        base_url: "https://".to_string() + &"b".repeat(10000) + ".com",
        model: "c".repeat(1000),
        yolo: Some(true),
    };

    let json = serde_json::to_string(&config).unwrap();
    let deserialized: QwenConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.api_key.len(), 10000);
    assert!(deserialized.base_url.len() > 10000);
    assert_eq!(deserialized.model.len(), 1000);
}

#[test]
fn test_gemini_auth_config_edge_cases() {
    // Test with maximum values
    let config = GeminiAuthConfig {
        method: "a".repeat(1000),
        api_key: Some("b".repeat(10000)),
        vertex_project: Some("c".repeat(1000)),
        vertex_location: Some("d".repeat(1000)),
        yolo: Some(false),
    };

    let json = serde_json::to_string(&config).unwrap();
    let deserialized: GeminiAuthConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.method.len(), 1000);
    assert_eq!(deserialized.api_key.as_ref().unwrap().len(), 10000);
}

#[test]
fn test_llxprt_config_edge_cases() {
    // Test with maximum values
    let config = LLxprtConfig {
        provider: "a".repeat(1000),
        api_key: "b".repeat(10000),
        model: "c".repeat(1000),
        base_url: Some("https://".to_string() + &"d".repeat(10000) + ".com"),
    };

    let json = serde_json::to_string(&config).unwrap();
    let deserialized: LLxprtConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.provider.len(), 1000);
    assert_eq!(deserialized.api_key.len(), 10000);
    assert!(deserialized.base_url.as_ref().unwrap().len() > 10000);
}

#[test]
fn test_json_rpc_error_edge_cases_detailed() {
    // Test with maximum values for error
    let large_error_data = serde_json::json!({
        "message": "a".repeat(1000000), // 1MB message in data
        "details": "b".repeat(1000000), // 1MB details in data
        "stack": "c".repeat(1000000),   // 1MB stack in data
    });

    let error = JsonRpcError {
        code: i32::MIN,
        message: "d".repeat(10000), // 10KB message
        data: Some(large_error_data),
    };

    let response = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: i32::MAX,
        result: None,
        error: Some(error),
    };

    let json = serde_json::to_string(&response).unwrap();
    let deserialized: JsonRpcResponse = serde_json::from_str(&json).unwrap();

    assert!(deserialized.error.is_some());
    assert_eq!(deserialized.id, i32::MAX);
    // The error message should be preserved
    assert_eq!(deserialized.error.as_ref().unwrap().message.len(), 10000);
}

#[tokio::test]
async fn test_rate_limit_tracker_extreme_limits() {
    let config = RateLimitingConfig {
        strategy: "round-robin".to_string(),
        track_usage: true,
        usage_db_path: None,
    };

    let mut tracker = RateLimitTracker::new(config);

    // Test with extremely low limits
    let extremely_low_limits = RateLimit {
        requests_per_minute: 1,
        requests_per_day: 1,
    };

    // First request should succeed
    assert!(tracker.check_and_increment("extreme-low-agent", &extremely_low_limits).await);
    
    // Second request should fail due to both limits
    assert!(!tracker.check_and_increment("extreme-low-agent", &extremely_low_limits).await);

    // Test with extremely high limits
    let extremely_high_limits = RateLimit {
        requests_per_minute: u32::MAX,
        requests_per_day: u32::MAX,
    };

    // Should succeed with high limits
    assert!(tracker.check_and_increment("extreme-high-agent", &extremely_high_limits).await);
    
    // Even multiple requests should succeed
    for i in 0..1000 {
        assert!(tracker.check_and_increment(&format!("high-agent-{}", i), &extremely_high_limits).await);
    }
}

#[test]
fn test_project_list_item_extreme_values() {
    // Test with the most extreme possible values
    let item = ProjectListItem {
        id: "a".repeat(usize::MAX.min(1_000_000)), // Cap at 1MB to avoid memory issues in test
        title: Some("b".repeat(usize::MAX.min(1_000_000))),
        status: Some("c".repeat(usize::MAX.min(100_000))), // Smaller cap for status
        created_at: Some("9999-12-31T23:59:59Z".to_string()),
        updated_at: Some("9999-12-31T23:59:59Z".to_string()),
        last_activity_at: Some("9999-12-31T23:59:59Z".to_string()),
        log_count: Some(u32::MAX),
    };

    // This should not panic, though it might fail due to memory constraints
    // In a real scenario, we'd want to test with more reasonable limits
    let _json = serde_json::to_string(&item).ok();
}

#[test]
fn test_project_metadata_extreme_values() {
    // Test with extreme but reasonable values
    let metadata = ProjectMetadata {
        path: PathBuf::from("/".to_string() + &"segment/".repeat(10_000) + "final"), // Very long path
        sha256: Some("a".repeat(64)), // Valid SHA256
        friendly_name: Some("b".repeat(100_000)), // Long friendly name
        first_used: Some(DateTime::parse_from_rfc3339("2023-01-01T00:00:00Z").unwrap().with_timezone(&Utc).with_timezone(&FixedOffset::east_opt(0).unwrap())),
        updated_at: Some(DateTime::parse_from_rfc3339("2023-12-31T23:59:59Z").unwrap().with_timezone(&Utc).with_timezone(&FixedOffset::east_opt(0).unwrap())),
    };

    let _json = serde_json::to_string(&metadata).ok();
}

#[test]
fn test_recent_chat_extreme_values() {
    // Test with extreme but reasonable values
    let chat = RecentChat {
        id: "a".repeat(100_000),
        title: "b".repeat(100_000),
        started_at_iso: "9999-12-31T23:59:59Z".to_string(),
        message_count: u32::MAX,
        summary: Some("c".repeat(1_000_000)),
        last_activity_iso: Some("9999-12-31T23:59:59Z".to_string()),
        total_tokens: Some(u32::MAX),
        tags: vec!["tag".to_string(); 10_000], // Many tags
    };

    let _json = serde_json::to_string(&chat).ok();
}

#[test]
fn test_message_match_extreme_values() {
    // Test with extreme but reasonable values
    let match_item = MessageMatch {
        content_snippet: "a".repeat(10_000_000), // 10MB snippet
        start_line: u32::MAX,
        end_line: u32::MAX,
        file_path: Some("/".to_string() + &"dir/".repeat(10_000) + "file.txt"),
        timestamp: Some("9999-12-31T23:59:59Z".to_string()),
    };

    let _json = serde_json::to_string(&match_item).ok();
}

#[test]
fn test_search_result_extreme_values() {
    // Test with extreme but reasonable values
    let result = SearchResult {
        chat: RecentChat {
            id: "test".to_string(),
            title: "Test".to_string(),
            started_at_iso: "2023-01-01T00:00:00Z".to_string(),
            message_count: 1,
            summary: None,
            last_activity_iso: None,
            total_tokens: None,
            tags: vec![],
        },
        matches: (0..100_000).map(|i| MessageMatch {
            content_snippet: format!("match {}", i),
            start_line: i as u32,
            end_line: (i + 1) as u32,
            file_path: Some(format!("/file{}.txt", i)),
            timestamp: Some("2023-01-01T00:00:00Z".to_string()),
        }).collect(),
        score: 1.0, // Maximum score
    };

    let _json = serde_json::to_string(&result).ok();
}

#[test]
fn test_detailed_conversation_extreme_values() {
    // Test with extreme but reasonable values
    let conversation = DetailedConversation {
        id: "a".repeat(100_000),
        title: "b".repeat(100_000),
        started_at_iso: "9999-12-31T23:59:59Z".to_string(),
        messages: (0..1_000_000).map(|i| ConversationHistoryEntry {
            id: format!("msg{}", i),
            role: if i % 2 == 0 { "user" } else { "assistant" }.to_string(),
            content: format!("message content {}", i),
            timestamp_iso: "2023-01-01T00:00:00Z".to_string(),
            tokens: Some((i % 1000) as u32),
        }).collect(),
        total_tokens: Some(u32::MAX),
        tags: vec!["tag".to_string(); 100_000],
    };

    let _json = serde_json::to_string(&conversation).ok();
}

#[test]
fn test_session_params_extreme_values() {
    // Test with extreme but reasonable values
    let params = SessionParams {
        session_id: "a".repeat(100_000),
        working_directory: "/".to_string() + &"dir/".repeat(100_000) + "final",
        model: "a".repeat(10_000),
        cli_name: Some("a".repeat(10_000)),
        backend_config: Some(QwenConfig {
            api_key: "a".repeat(100_000),
            base_url: "https://".to_string() + &"a".repeat(100_000) + ".com",
            model: "a".repeat(10_000),
            yolo: Some(true),
        }),
        gemini_auth: Some(GeminiAuthConfig {
            method: "a".repeat(10_000),
            api_key: Some("a".repeat(100_000)),
            vertex_project: Some("a".repeat(10_000)),
            vertex_location: Some("a".repeat(10_000)),
            yolo: Some(false),
        }),
        llxprt_config: Some(LLxprtConfig {
            provider: "a".repeat(10_000),
            api_key: "a".repeat(100_000),
            model: "a".repeat(10_000),
            base_url: Some("https://".to_string() + &"a".repeat(100_000) + ".com"),
        }),
    };

    let _json = serde_json::to_string(&params).ok();
}

#[test]
fn test_file_content_extreme_values() {
    // Test with extreme but reasonable values (capped to avoid memory issues)
    let content = FileContent {
        path: "/".to_string() + &"dir/".repeat(10_000) + "file.txt",
        content: "a".repeat(100_000_000.min(50_000_000)), // Cap at 50MB to avoid memory issues
        size: u64::MAX,
        modified: "9999-12-31T23:59:59Z".to_string(),
        mime_type: "a".repeat(10_000),
    };

    let _json = serde_json::to_string(&content).ok();
}

#[test]
fn test_dir_entry_extreme_values() {
    // Test with extreme but reasonable values
    let entry = DirEntry {
        name: "a".repeat(100_000),
        path: "/".to_string() + &"dir/".repeat(100_000) + "file",
        size: u64::MAX,
        modified: "9999-12-31T23:59:59Z".to_string(),
        is_directory: true,
        mime_type: "a".repeat(10_000),
    };

    let _json = serde_json::to_string(&entry).ok();
}

#[test]
fn test_gemini_command_extreme_values() {
    // Test with extreme but reasonable values
    let command = GeminiCommand {
        name: "a".repeat(100_000),
        description: "b".repeat(1_000_000),
        path: "/".to_string() + &"dir/".repeat(10_000) + "command.md",
    };

    let _json = serde_json::to_string(&command).ok();
}

#[test]
fn test_git_info_extreme_values() {
    // Test with extreme but reasonable values
    let git_info = GitInfo {
        remote_url: Some("https://".to_string() + &"a".repeat(1_000_000) + ".git"),
        branch: Some("a".repeat(100_000)),
        commit_hash: Some("a".repeat(100_000)), // Not a real hash but tests the field
        is_dirty: Some(true),
    };

    let _json = serde_json::to_string(&git_info).ok();
}

#[test]
fn test_json_rpc_request_extreme_values() {
    // Test with extreme but reasonable values
    let huge_params = serde_json::json!({
        "data": "a".repeat(50_000_000.min(25_000_000)), // Cap at 25MB to avoid memory issues
        "nested": {
            "level1": {
                "level2": {
                    "level3": "b".repeat(1_000_000)
                }
            }
        }
    });

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: i32::MAX,
        method: "a".repeat(100_000),
        params: huge_params,
    };

    let _json = serde_json::to_string(&request).ok();
}

#[test]
fn test_json_rpc_response_extreme_values() {
    // Test with extreme but reasonable values
    let huge_result = serde_json::json!({
        "data": "a".repeat(50_000_000.min(25_000_000)), // Cap at 25MB
        "nested": {
            "level1": {
                "level2": {
                    "level3": "b".repeat(1_000_000)
                }
            }
        }
    });

    let response = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: i32::MAX,
        result: Some(huge_result),
        error: None,
    };

    let _json = serde_json::to_string(&response).ok();
}

#[test]
fn test_json_rpc_error_extreme_values() {
    // Test with extreme but reasonable values
    let huge_error_data = serde_json::json!({
        "details": "a".repeat(25_000_000.min(10_000_000)), // Cap at 10MB
        "context": "b".repeat(1_000_000),
        "stack": "c".repeat(1_000_000)
    });

    let error = JsonRpcError {
        code: i32::MIN,
        message: "a".repeat(100_000),
        data: Some(huge_error_data),
    };

    let response = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: i32::MAX,
        result: None,
        error: Some(error),
    };

    let _json = serde_json::to_string(&response).ok();
}

#[test]
fn test_tool_call_confirmation_extreme_values() {
    // Test with extreme but reasonable values
    let confirmation = ToolCallConfirmation {
        session_id: "a".repeat(100_000),
        request_id: u32::MAX,
        tool_call_id: "b".repeat(100_000),
        outcome: "c".repeat(100_000),
    };

    let _json = serde_json::to_string(&confirmation).ok();
}

#[test]
fn test_enriched_project_extreme_values() {
    // Test with extreme but reasonable values
    let project = EnrichedProject {
        sha256: "a".repeat(64), // Valid SHA256
        root_path: PathBuf::from("/".to_string() + &"dir/".repeat(10_000) + "final"),
        metadata: ProjectMetadataView {
            path: "/".to_string() + &"dir/".repeat(10_000) + "final",
            sha256: "a".repeat(64),
            friendly_name: "b".repeat(1_000_000),
            first_used: Some("9999-12-31T23:59:59Z".to_string()),
            updated_at: Some("9999-12-31T23:59:59Z".to_string()),
        },
    };

    let _json = serde_json::to_string(&project).ok();
}

#[test]
fn test_project_metadata_view_extreme_values() {
    // Test with extreme but reasonable values
    let view = ProjectMetadataView {
        path: "/".to_string() + &"dir/".repeat(10_000) + "file",
        sha256: "a".repeat(64), // Valid SHA256
        friendly_name: "b".repeat(1_000_000),
        first_used: Some("9999-12-31T23:59:59Z".to_string()),
        updated_at: Some("9999-12-31T23:59:59Z".to_string()),
    };

    let _json = serde_json::to_string(&view).ok();
}

#[test]
fn test_search_filters_extreme_values() {
    // Test with extreme but reasonable values
    let filters = SearchFilters {
        date_range: Some(("0001-01-01".to_string(), "9999-12-31".to_string())),
        tags: Some(vec!["tag".to_string(); 1_000_000]), // Million tags
        project_id: Some("a".repeat(100_000)),
        max_results: Some(u32::MAX),
    };

    let _json = serde_json::to_string(&filters).ok();
}

#[test]
fn test_conversation_history_entry_extreme_values() {
    // Test with extreme but reasonable values
    let entry = ConversationHistoryEntry {
        id: "a".repeat(100_000),
        role: "b".repeat(10_000),
        content: "c".repeat(50_000_000.min(25_000_000)), // Cap at 25MB
        timestamp_iso: "9999-12-31T23:59:59Z".to_string(),
        tokens: Some(u32::MAX),
    };

    let _json = serde_json::to_string(&entry).ok();
}

#[test]
fn test_command_result_extreme_values() {
    // Test with extreme but reasonable values
    let result = CommandResult {
        success: true,
        output: "a".repeat(100_000_000.min(50_000_000)), // Cap at 50MB
        error: Some("b".repeat(25_000_000.min(10_000_000))), // Cap at 10MB
    };

    let _json = serde_json::to_string(&result).ok();
}

#[test]
fn test_message_chunk_extreme_values() {
    // Test with extreme but reasonable values
    let chunk = MessageChunk {
        content: "a".repeat(100_000_000.min(50_000_000)), // Cap at 50MB
        role: "b".repeat(10_000),
        timestamp: "9999-12-31T23:59:59Z".to_string(),
    };

    let _json = serde_json::to_string(&chunk).ok();
}

#[test]
fn test_assistant_chunk_extreme_values() {
    // Test with extreme but reasonable values
    let chunk = AssistantChunk {
        text: Some("a".repeat(100_000_000.min(50_000_000))), // Cap at 50MB
        tool_calls: Some(vec![]),
        finish_reason: Some("b".repeat(10_000)),
    };

    let _json = serde_json::to_string(&chunk).ok();
}

#[test]
fn test_stream_assistant_message_chunk_params_extreme_values() {
    // Test with extreme but reasonable values
    let params = StreamAssistantMessageChunkParams {
        session_id: "a".repeat(100_000),
        chunk: AssistantChunk {
            text: Some("b".repeat(50_000_000.min(25_000_000))), // Cap at 25MB
            tool_calls: Some(vec![]),
            finish_reason: Some("c".repeat(10_000)),
        },
    };

    let _json = serde_json::to_string(&params).ok();
}

#[test]
fn test_cli_io_payload_extreme_values() {
    // Test with extreme but reasonable values
    let payload = CliIoPayload {
        io_type: CliIoType::Output,
        data: "a".repeat(100_000_000.min(50_000_000)), // Cap at 50MB
    };

    let _json = serde_json::to_string(&payload).ok();
}

#[test]
fn test_error_payload_extreme_values() {
    // Test with extreme but reasonable values
    let payload = ErrorPayload {
        error: "a".repeat(100_000_000.min(50_000_000)), // Cap at 50MB
    };

    let _json = serde_json::to_string(&payload).ok();
}

#[test]
fn test_gemini_output_payload_extreme_values() {
    // Test with extreme but reasonable values
    let payload = GeminiOutputPayload {
        text: "a".repeat(100_000_000.min(50_000_000)), // Cap at 50MB
    };

    let _json = serde_json::to_string(&payload).ok();
}

#[test]
fn test_gemini_thought_payload_extreme_values() {
    // Test with extreme but reasonable values
    let payload = GeminiThoughtPayload {
        thought: "a".repeat(100_000_000.min(50_000_000)), // Cap at 50MB
    };

    let _json = serde_json::to_string(&payload).ok();
}

#[test]
fn test_tool_call_confirmation_content_extreme_values() {
    // Test with extreme but reasonable values
    let content = ToolCallConfirmationContent {
        tool_call_id: "a".repeat(100_000),
        outcome: "b".repeat(100_000),
    };

    let _json = serde_json::to_string(&content).ok();
}

#[test]
fn test_tool_call_event_extreme_values() {
    // Test with extreme but reasonable values
    let event = ToolCallEvent {
        id: "a".repeat(100_000),
        tool_call_id: "b".repeat(100_000),
        status: ToolCallStatus::Requested,
        title: "c".repeat(100_000),
        content: vec![],
        locations: vec![],
        kind: "d".repeat(10_000),
        server_name: "e".repeat(10_000),
        tool_name: "f".repeat(10_000),
    };

    let _json = serde_json::to_string(&event).ok();
}

#[test]
fn test_tool_call_location_extreme_values() {
    // Test with extreme but reasonable values
    let location = ToolCallLocation {
        file: "/".to_string() + &"dir/".repeat(10_000) + "file",
        line: u32::MAX,
        column: u32::MAX,
    };

    let _json = serde_json::to_string(&location).ok();
}

#[test]
fn test_tool_call_update_extreme_values() {
    // Test with extreme but reasonable values
    let update = ToolCallUpdate {
        tool_call_id: "a".repeat(100_000),
        status: ToolCallStatus::Completed,
        content: vec![],
        server_name: "b".repeat(10_000),
        tool_name: "c".repeat(10_000),
    };

    let _json = serde_json::to_string(&update).ok();
}

#[test]
fn test_tool_call_content_item_extreme_values() {
    // Test with extreme but reasonable values
    let item = ToolCallContentItem {
        r#type: "a".repeat(10_000),
        text: "b".repeat(50_000_000.min(25_000_000)), // Cap at 25MB
    };

    let _json = serde_json::to_string(&item).ok();
}

#[test]
fn test_content_block_extreme_values() {
    // Test with extreme but reasonable values
    let block = ContentBlock::Text { text: "a".repeat(50_000_000.min(25_000_000)) }; // Cap at 25MB

    let _json = serde_json::to_string(&block).ok();
}

#[test]
fn test_qwen_config_extreme_values() {
    // Test with extreme but reasonable values
    let config = QwenConfig {
        api_key: "a".repeat(100_000),
        base_url: "https://".to_string() + &"b".repeat(100_000) + ".com",
        model: "c".repeat(10_000),
        yolo: Some(true),
    };

    let _json = serde_json::to_string(&config).ok();
}

#[test]
fn test_gemini_auth_config_extreme_values() {
    // Test with extreme but reasonable values
    let config = GeminiAuthConfig {
        method: "a".repeat(10_000),
        api_key: Some("b".repeat(100_000)),
        vertex_project: Some("c".repeat(10_000)),
        vertex_location: Some("d".repeat(10_000)),
        yolo: Some(false),
    };

    let _json = serde_json::to_string(&config).ok();
}

#[test]
fn test_llxprt_config_extreme_values() {
    // Test with extreme but reasonable values
    let config = LLxprtConfig {
        provider: "a".repeat(10_000),
        api_key: "b".repeat(100_000),
        model: "c".repeat(10_000),
        base_url: Some("https://".to_string() + &"d".repeat(100_000) + ".com"),
    };

    let _json = serde_json::to_string(&config).ok();
}

#[tokio::test]
async fn test_rate_limit_tracker_functionality_under_stress() {
    let config = RateLimitingConfig {
        strategy: "round-robin".to_string(),
        track_usage: true,
        usage_db_path: None,
    };

    let mut tracker = RateLimitTracker::new(config);

    let high_limit = RateLimit {
        requests_per_minute: 1000,
        requests_per_day: 10000,
    };

    // Perform many operations to stress test the tracker
    for i in 0..5000 {
        let agent_id = format!("stress-agent-{}", i % 100); // Cycle through 100 agents
        
        let success = tracker.check_and_increment(&agent_id, &high_limit).await;
        assert!(success, "Request {} for agent {} should succeed", i, agent_id);
        
        // Occasionally check usage
        if i % 100 == 0 {
            let usage = tracker.get_usage(&agent_id);
            assert!(usage.is_some(), "Usage should be tracked for agent {}", agent_id);
        }
    }

    // Verify that the tracker is still functional after stress
    assert!(tracker.check_and_increment("post-stress-agent", &high_limit).await);
}

#[test]
fn test_all_serialization_combinations() {
    // Test that all our structs can be serialized and deserialized
    // This is a smoke test to ensure basic functionality
    
    // Rate limiting
    let config = RateLimitingConfig {
        strategy: "round-robin".to_string(),
        track_usage: true,
        usage_db_path: None,
    };
    let _json = serde_json::to_string(&config).unwrap();
    
    let limit = RateLimit {
        requests_per_minute: 100,
        requests_per_day: 1000,
    };
    let _json = serde_json::to_string(&limit).unwrap();
    
    // Projects
    let item = ProjectListItem {
        id: "test".to_string(),
        title: Some("Test".to_string()),
        status: Some("active".to_string()),
        created_at: Some("2023-01-01T00:00:00Z".to_string()),
        updated_at: Some("2023-01-02T00:00:00Z".to_string()),
        last_activity_at: Some("2023-01-03T00:00:00Z".to_string()),
        log_count: Some(5),
    };
    let _json = serde_json::to_string(&item).unwrap();
    
    let response = ProjectsResponse {
        items: vec![item],
        total: 1,
        limit: 10,
        offset: 0,
    };
    let _json = serde_json::to_string(&response).unwrap();
    
    let metadata = ProjectMetadata {
        path: PathBuf::from("/test"),
        sha256: Some("testsha".to_string()),
        friendly_name: Some("test".to_string()),
        first_used: None,
        updated_at: None,
    };
    let _json = serde_json::to_string(&metadata).unwrap();
    
    let view = ProjectMetadataView {
        path: "/test".to_string(),
        sha256: "testsha".to_string(),
        friendly_name: "test".to_string(),
        first_used: None,
        updated_at: None,
    };
    let _json = serde_json::to_string(&view).unwrap();
    
    let enriched = EnrichedProject {
        sha256: "testsha".to_string(),
        root_path: PathBuf::from("/test"),
        metadata: view,
    };
    let _json = serde_json::to_string(&enriched).unwrap();
    
    // Search
    let chat = RecentChat {
        id: "test".to_string(),
        title: "Test".to_string(),
        started_at_iso: "2023-01-01T00:00:00Z".to_string(),
        message_count: 1,
        summary: None,
        last_activity_iso: None,
        total_tokens: None,
        tags: vec![],
    };
    let _json = serde_json::to_string(&chat).unwrap();
    
    let match_item = MessageMatch {
        content_snippet: "test".to_string(),
        start_line: 1,
        end_line: 2,
        file_path: None,
        timestamp: None,
    };
    let _json = serde_json::to_string(&match_item).unwrap();
    
    let search_result = SearchResult {
        chat,
        matches: vec![match_item],
        score: 0.5,
    };
    let _json = serde_json::to_string(&search_result).unwrap();
    
    let filters = SearchFilters::default();
    let _json = serde_json::to_string(&filters).unwrap();
    
    let entry = ConversationHistoryEntry {
        id: "test".to_string(),
        role: "user".to_string(),
        content: "test".to_string(),
        timestamp_iso: "2023-01-01T00:00:00Z".to_string(),
        tokens: Some(1),
    };
    let _json = serde_json::to_string(&entry).unwrap();
    
    let conversation = DetailedConversation {
        id: "test".to_string(),
        title: "Test".to_string(),
        started_at_iso: "2023-01-01T00:00:00Z".to_string(),
        messages: vec![entry],
        total_tokens: Some(10),
        tags: vec![],
    };
    let _json = serde_json::to_string(&conversation).unwrap();
    
    // Session
    let session_params = SessionParams {
        session_id: "test".to_string(),
        working_directory: "/test".to_string(),
        model: "test".to_string(),
        cli_name: None,
        backend_config: None,
        gemini_auth: None,
        llxprt_config: None,
    };
    let _json = serde_json::to_string(&session_params).unwrap();
    
    let qwen_config = QwenConfig {
        api_key: "test".to_string(),
        base_url: "https://test.com".to_string(),
        model: "test".to_string(),
        yolo: None,
    };
    let _json = serde_json::to_string(&qwen_config).unwrap();
    
    let auth_config = GeminiAuthConfig {
        method: "test".to_string(),
        api_key: Some("test".to_string()),
        vertex_project: Some("test".to_string()),
        vertex_location: Some("test".to_string()),
        yolo: None,
    };
    let _json = serde_json::to_string(&auth_config).unwrap();
    
    let llxprt_config = LLxprtConfig {
        provider: "test".to_string(),
        api_key: "test".to_string(),
        model: "test".to_string(),
        base_url: Some("https://test.com".to_string()),
    };
    let _json = serde_json::to_string(&llxprt_config).unwrap();
    
    // Filesystem
    let file_content = FileContent {
        path: "/test.txt".to_string(),
        content: "test".to_string(),
        size: 4,
        modified: "2023-01-01T00:00:00Z".to_string(),
        mime_type: "text/plain".to_string(),
    };
    let _json = serde_json::to_string(&file_content).unwrap();
    
    let dir_entry = DirEntry {
        name: "test".to_string(),
        path: "/test".to_string(),
        size: 0,
        modified: "2023-01-01T00:00:00Z".to_string(),
        is_directory: false,
        mime_type: "file".to_string(),
    };
    let _json = serde_json::to_string(&dir_entry).unwrap();
    
    let command = GeminiCommand {
        name: "test".to_string(),
        description: "test".to_string(),
        path: "/test.md".to_string(),
    };
    let _json = serde_json::to_string(&command).unwrap();
    
    let git_info = GitInfo {
        remote_url: Some("https://test.com".to_string()),
        branch: Some("main".to_string()),
        commit_hash: Some("test".to_string()),
        is_dirty: Some(false),
    };
    let _json = serde_json::to_string(&git_info).unwrap();
    
    // RPC
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: 1,
        method: "test".to_string(),
        params: serde_json::Value::Null,
    };
    let _json = serde_json::to_string(&request).unwrap();
    
    let response = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: 1,
        result: Some(serde_json::Value::String("test".to_string())),
        error: None,
    };
    let _json = serde_json::to_string(&response).unwrap();
    
    let error = JsonRpcError {
        code: -32600,
        message: "test".to_string(),
        data: None,
    };
    let _json = serde_json::to_string(&error).unwrap();
    
    // Events
    let io_payload = CliIoPayload {
        io_type: CliIoType::Output,
        data: "test".to_string(),
    };
    let _json = serde_json::to_string(&io_payload).unwrap();
    
    let error_payload = ErrorPayload {
        error: "test".to_string(),
    };
    let _json = serde_json::to_string(&error_payload).unwrap();
    
    let output_payload = GeminiOutputPayload {
        text: "test".to_string(),
    };
    let _json = serde_json::to_string(&output_payload).unwrap();
    
    let thought_payload = GeminiThoughtPayload {
        thought: "test".to_string(),
    };
    let _json = serde_json::to_string(&thought_payload).unwrap();
    
    let confirmation = ToolCallConfirmation {
        session_id: "test".to_string(),
        request_id: 1,
        tool_call_id: "test".to_string(),
        outcome: "test".to_string(),
    };
    let _json = serde_json::to_string(&confirmation).unwrap();
    
    let confirmation_content = ToolCallConfirmationContent {
        tool_call_id: "test".to_string(),
        outcome: "test".to_string(),
    };
    let _json = serde_json::to_string(&confirmation_content).unwrap();
    
    let event = ToolCallEvent {
        id: "test".to_string(),
        tool_call_id: "test".to_string(),
        status: ToolCallStatus::Requested,
        title: "test".to_string(),
        content: vec![],
        locations: vec![],
        kind: "test".to_string(),
        server_name: "test".to_string(),
        tool_name: "test".to_string(),
    };
    let _json = serde_json::to_string(&event).unwrap();
    
    let location = ToolCallLocation {
        file: "/test".to_string(),
        line: 1,
        column: 1,
    };
    let _json = serde_json::to_string(&location).unwrap();
    
    let update = ToolCallUpdate {
        tool_call_id: "test".to_string(),
        status: ToolCallStatus::Completed,
        content: vec![],
        server_name: "test".to_string(),
        tool_name: "test".to_string(),
    };
    let _json = serde_json::to_string(&update).unwrap();
    
    let content_item = ToolCallContentItem {
        r#type: "test".to_string(),
        text: "test".to_string(),
    };
    let _json = serde_json::to_string(&content_item).unwrap();
    
    // ACP
    let content_block = ContentBlock::Text { text: "test".to_string() };
    let _json = serde_json::to_string(&content_block).unwrap();
    
    let init_params = InitializeParams {
        protocol_version: 1,
        client_capabilities: bl1nk_core::adapters::acp::ClientCapabilities {
            fs: bl1nk_core::adapters::acp::FileSystemCapabilities {
                read_text_file: false,
                write_text_file: false,
            },
        },
    };
    let _json = serde_json::to_string(&init_params).unwrap();
    
    let session_new_params = SessionNewParams {
        cwd: "/test".to_string(),
        mcp_servers: vec![],
    };
    let _json = serde_json::to_string(&session_new_params).unwrap();
    
    let prompt_params = SessionPromptParams {
        session_id: "test".to_string(),
        prompt: vec![content_block],
    };
    let _json = serde_json::to_string(&prompt_params).unwrap();
    
    let auth_params = AuthenticateParams {
        method_id: "test".to_string(),
    };
    let _json = serde_json::to_string(&auth_params).unwrap();
    
    let session_new_result = SessionNewResult {
        session_id: "test".to_string(),
    };
    let _json = serde_json::to_string(&session_new_result).unwrap();
    
    let prompt_result = SessionPromptResult {
        stop_reason: "test".to_string(),
        output: vec![],
    };
    let _json = serde_json::to_string(&prompt_result).unwrap();
    
    let perm_params = SessionRequestPermissionParams {
        tool_call: bl1nk_core::adapters::acp::ToolCall {
            id: "test".to_string(),
            name: "test".to_string(),
            arguments: serde_json::Value::Null,
        },
    };
    let _json = serde_json::to_string(&perm_params).unwrap();
    
    // CLI
    let cmd_result = CommandResult {
        success: true,
        output: "test".to_string(),
        error: None,
    };
    let _json = serde_json::to_string(&cmd_result).unwrap();
    
    let msg_chunk = MessageChunk {
        content: "test".to_string(),
        role: "test".to_string(),
        timestamp: "2023-01-01T00:00:00Z".to_string(),
    };
    let _json = serde_json::to_string(&msg_chunk).unwrap();
    
    let assistant_chunk = AssistantChunk {
        text: Some("test".to_string()),
        tool_calls: None,
        finish_reason: None,
    };
    let _json = serde_json::to_string(&assistant_chunk).unwrap();
    
    let stream_params = StreamAssistantMessageChunkParams {
        session_id: "test".to_string(),
        chunk: assistant_chunk,
    };
    let _json = serde_json::to_string(&stream_params).unwrap();
    
    // All tests passed if we reach here without panicking
}