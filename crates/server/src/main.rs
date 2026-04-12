use anyhow::{Error as AnyhowError, Result};
use bl1nk_core::{
    session, DetailedConversation, DirEntry, EnrichedProject, EventEmitter, FileContent,
    GeminiBackend, GeminiCommand, GitInfo, ProcessStatus, RecentChat, SearchFilters, SearchResult,
};
use bl1nk_core::config::Config;
use bl1nk_core::mcp::Orchestrator;
use clap::Parser;
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

use include_dir::{include_dir, Dir};
use rocket::{
    get,
    http::{ContentType, Status},
    post,
    response::{self, Responder},
    routes,
    serde::json::Json,
    Request, Response, Shutdown, State,
};
use rocket_ws::{Message, Stream, WebSocket};
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use std::sync::mpsc;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use tokio::sync::{mpsc as tokio_mpsc, Mutex};

// Mock frontend dir if not exists to allow compilation
static FRONTEND_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/../../agents"); // Using agents dir as placeholder for now

// --- CLI Args ---
#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    config: Option<PathBuf>,
    #[arg(short, long, default_value = "info")]
    log_level: String,
}

// =====================================
// WebSocket Connection Management
// =====================================

#[derive(Clone)]
pub struct WebSocketManager {
    connections: Arc<Mutex<Vec<tokio_mpsc::UnboundedSender<String>>>>,
    connection_counter: Arc<AtomicU64>,
}

impl Default for WebSocketManager {
    fn default() -> Self {
        Self {
            connections: Arc::new(Mutex::new(Vec::new())),
            connection_counter: Arc::new(AtomicU64::new(0)),
        }
    }
}

impl WebSocketManager {
    pub fn new() -> Self {
        Self::default()
    }
    pub async fn add_connection(&self, sender: tokio_mpsc::UnboundedSender<String>) -> u64 {
        let connection_id = self.connection_counter.fetch_add(1, Ordering::SeqCst);
        self.connections.lock().await.push(sender);
        connection_id
    }
    pub async fn remove_connection(&self, sender: &tokio_mpsc::UnboundedSender<String>) {
        let mut connections = self.connections.lock().await;
        if let Some(pos) = connections
            .iter()
            .position(|conn| std::ptr::eq(conn, sender))
        {
            connections.remove(pos);
        }
    }
    pub async fn broadcast(&self, message: String) -> anyhow::Result<()> {
        let mut connections = self.connections.lock().await;
        connections.retain(|sender| sender.send(message.clone()).is_ok());
        Ok(())
    }
}

#[derive(Serialize)]
struct WebSocketEvent<T> {
    event: String,
    payload: T,
    sequence: u64,
}

#[derive(Clone)]
pub struct WebSocketsEventEmitter {
    sequence_counter: Arc<AtomicU64>,
    event_sender: mpsc::Sender<String>,
}

impl WebSocketsEventEmitter {
    pub fn new(ws_manager: WebSocketManager) -> Self {
        let (event_sender, event_receiver) = mpsc::channel::<String>();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async move {
                while let Ok(message) = event_receiver.recv() {
                    let _ = ws_manager.broadcast(message).await;
                }
            });
        });
        Self {
            sequence_counter: Arc::new(AtomicU64::new(0)),
            event_sender,
        }
    }
}

impl EventEmitter for WebSocketsEventEmitter {
    fn emit<S: Serialize + Clone>(&self, event: &str, payload: S) -> Result<()> {
        let sequence = self.sequence_counter.fetch_add(1, Ordering::SeqCst);
        let ws_event = WebSocketEvent {
            event: event.to_string(),
            payload,
            sequence,
        };
        let message = serde_json::to_string(&ws_event)?;
        self.event_sender.send(message)?;
        Ok(())
    }
}

struct AppState {
    backend: Arc<Mutex<GeminiBackend<WebSocketsEventEmitter>>>,
    ws_manager: WebSocketManager,
    #[allow(dead_code)]
    orchestrator: Arc<Orchestrator>, // Added Orchestrator to state
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StartSessionRequest {
    session_id: String,
    working_directory: Option<String>,
    model: Option<String>,
    cli_name: Option<String>,
    backend_config: Option<session::QwenConfig>,
    gemini_auth: Option<session::GeminiAuthConfig>,
    llxprt_config: Option<session::LLxprtConfig>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SendMessageRequest {
    session_id: String,
    message: String,
    conversation_history: String,
    model: Option<String>,
    backend_config: Option<session::QwenConfig>,
    gemini_auth: Option<session::GeminiAuthConfig>,
}

#[derive(Serialize, Deserialize)]
struct KillProcessRequest {
    conversation_id: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ToolConfirmationRequest {
    session_id: String,
    request_id: u32,
    tool_call_id: String,
    outcome: String,
}

#[derive(Serialize, Deserialize)]
struct ExecuteCommandRequest {
    command: String,
}

#[derive(Serialize, Deserialize)]
struct ApproveOauthRequest {
    approved: bool,
}
#[derive(Serialize, Deserialize)]
struct GenerateTitleRequest {
    message: String,
    model: Option<String>,
}
#[derive(Serialize, Deserialize)]
struct ValidateDirectoryRequest {
    path: String,
}
#[derive(Serialize, Deserialize)]
struct IsHomeDirectoryRequest {
    path: String,
}
#[derive(Serialize, Deserialize)]
struct ListDirectoryRequest {
    path: String,
}
#[derive(Serialize, Deserialize)]
struct ListFilesRecursiveRequest {
    path: String,
}
#[derive(Serialize, Deserialize)]
struct GetParentDirectoryRequest {
    path: String,
}
#[derive(Serialize, Deserialize)]
struct GetGitInfoRequest {
    path: String,
}
#[derive(Serialize, Deserialize)]
struct ReadFileContentRequest {
    path: String,
}
#[derive(Serialize, Deserialize)]
struct ReadBinaryFileAsBase64Request {
    path: String,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ListGeminiCommandsRequest {
    working_directory: String,
}
#[derive(Serialize, Deserialize)]
struct CanonicalPathRequest {
    path: String,
}
#[derive(Serialize, Deserialize)]
struct ReadFileContentWithOptionsRequest {
    path: String,
    force_text: bool,
}
#[derive(Serialize, Deserialize)]
struct WriteFileContentRequest {
    path: String,
    content: String,
}

#[derive(Debug)]
pub struct AnyhowResponder(pub AnyhowError);
impl<'r> Responder<'r, 'static> for AnyhowResponder {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        let error_message = format!("{{\"error\":\"{:#}\"}}", self.0);
        Response::build()
            .status(Status::InternalServerError)
            .header(ContentType::JSON)
            .sized_body(error_message.len(), Cursor::new(error_message))
            .ok()
    }
}
impl<E> From<E> for AnyhowResponder
where
    E: Into<AnyhowError>,
{
    fn from(error: E) -> Self {
        AnyhowResponder(error.into())
    }
}
pub type AppResult<T> = std::result::Result<T, AnyhowResponder>;

#[get("/<path..>")]
fn index(path: PathBuf) -> Result<(ContentType, &'static [u8]), Status> {
    let file = FRONTEND_DIR
        .get_file(&path)
        .or_else(|| FRONTEND_DIR.get_file("index.html"))
        .ok_or(Status::NotFound)?;
    let content_type = path
        .extension()
        .and_then(|e| ContentType::from_extension(e.to_str().unwrap()))
        .unwrap_or(ContentType::HTML);
    Ok((content_type, file.contents()))
}

#[get("/projects?<limit>&<offset>")]
async fn list_projects(
    limit: Option<u32>,
    offset: Option<u32>,
    state: &State<AppState>,
) -> AppResult<Json<serde_json::Value>> {
    let backend = state.backend.lock().await;
    Ok(Json(
        backend
            .list_projects(limit.unwrap_or(25), offset.unwrap_or(0))
            .await?,
    ))
}

#[get("/projects-enriched")]
async fn list_enriched_projects(state: &State<AppState>) -> AppResult<Json<Vec<EnrichedProject>>> {
    let backend = state.backend.lock().await;
    Ok(Json(backend.list_enriched_projects().await?))
}

#[get("/project?<sha256>&<external_root_path>")]
async fn get_enriched_project_http(
    state: &State<AppState>,
    sha256: String,
    external_root_path: String,
) -> AppResult<Json<EnrichedProject>> {
    let backend = state.backend.lock().await;
    Ok(Json(
        backend
            .get_enriched_project(sha256, external_root_path)
            .await?,
    ))
}

#[get("/projects/<project_id>/discussions")]
async fn get_project_discussions(
    project_id: &str,
    state: &State<AppState>,
) -> AppResult<Json<Vec<RecentChat>>> {
    let backend = state.backend.lock().await;
    Ok(Json(backend.get_project_discussions(project_id).await?))
}

#[get("/recent-chats")]
async fn get_recent_chats(state: &State<AppState>) -> AppResult<Json<Vec<RecentChat>>> {
    let backend = state.backend.lock().await;
    Ok(Json(backend.get_recent_chats().await?))
}

#[derive(Deserialize)]
struct SearchChatsRequest {
    query: String,
    filters: Option<SearchFilters>,
}
#[post("/search-chats", data = "<request>")]
async fn search_chats(
    request: Json<SearchChatsRequest>,
    state: &State<AppState>,
) -> AppResult<Json<Vec<SearchResult>>> {
    let backend = state.backend.lock().await;
    Ok(Json(
        backend
            .search_chats(request.query.clone(), request.filters.clone())
            .await?,
    ))
}

#[get("/conversations/<chat_id>")]
async fn get_detailed_conversation(
    chat_id: String,
    state: &State<AppState>,
) -> AppResult<Json<DetailedConversation>> {
    let chat_id = urlencoding::decode(&chat_id).map_err(|e| AnyhowError::msg(e.to_string()))?;
    let backend = state.backend.lock().await;
    Ok(Json(backend.get_detailed_conversation(&chat_id).await?))
}

#[derive(Deserialize)]
struct ExportConversationRequest {
    format: String,
}
#[post("/conversations/<chat_id>/export", data = "<request>")]
async fn export_conversation_history(
    chat_id: String,
    request: Json<ExportConversationRequest>,
    state: &State<AppState>,
) -> AppResult<String> {
    let backend = state.backend.lock().await;
    Ok(backend
        .export_conversation_history(&chat_id, &request.format)
        .await?)
}

#[get("/check-cli-installed")]
async fn check_cli_installed(state: &State<AppState>) -> AppResult<Json<bool>> {
    let backend = state.backend.lock().await;
    Ok(Json(backend.check_cli_installed().await?))
}

#[get("/check-cli-installed/<name>")]
async fn check_cli_installed_named(name: String, state: &State<AppState>) -> AppResult<Json<bool>> {
    let backend = state.backend.lock().await;
    Ok(Json(backend.check_cli_installed_named(&name).await?))
}

#[post("/start-session", data = "<request>")]
async fn start_session(
    request: Json<StartSessionRequest>,
    state: &State<AppState>,
) -> AppResult<()> {
    let req = request.into_inner();
    let backend = state.backend.lock().await;
    if let Some(wd) = req.working_directory {
        backend
            .initialize_session(
                req.session_id,
                wd,
                req.model.unwrap_or_else(|| "gemini-2.0-flash-exp".into()),
                req.cli_name,
                req.backend_config,
                req.gemini_auth,
                req.llxprt_config,
            )
            .await?;
    }
    Ok(())
}

#[post("/start-session/<cli>", data = "<request>")]
async fn start_session_with_cli(
    cli: String,
    request: Json<StartSessionRequest>,
    state: &State<AppState>,
) -> AppResult<()> {
    let mut req = request.into_inner();
    req.cli_name = Some(cli);
    let backend = state.backend.lock().await;
    if let Some(wd) = req.working_directory {
        backend
            .initialize_session(
                req.session_id,
                wd,
                req.model.unwrap_or_else(|| "gemini-2.0-flash-exp".into()),
                req.cli_name,
                req.backend_config,
                req.gemini_auth,
                req.llxprt_config,
            )
            .await?;
    }
    Ok(())
}

#[post("/send-message", data = "<request>")]
async fn send_message(request: Json<SendMessageRequest>, state: &State<AppState>) -> AppResult<()> {
    let req = request.into_inner();
    let backend = state.backend.lock().await;
    backend
        .send_message(req.session_id, req.message, req.conversation_history)
        .await?;
    Ok(())
}

#[get("/process-statuses")]
async fn get_process_statuses(state: &State<AppState>) -> AppResult<Json<Vec<ProcessStatus>>> {
    let backend = state.backend.lock().await;
    Ok(Json(backend.get_process_statuses()?))
}

#[post("/kill-process", data = "<request>")]
async fn kill_process(request: Json<KillProcessRequest>, state: &State<AppState>) -> AppResult<()> {
    let backend = state.backend.lock().await;
    Ok(backend.kill_process(&request.conversation_id)?)
}

#[post("/tool-confirmation", data = "<request>")]
async fn send_tool_call_confirmation_response(
    request: Json<ToolConfirmationRequest>,
    state: &State<AppState>,
) -> AppResult<()> {
    let req = request.into_inner();
    let backend = state.backend.lock().await;
    Ok(backend
        .handle_tool_confirmation(
            req.session_id,
            req.request_id,
            req.tool_call_id,
            req.outcome,
        )
        .await?)
}

#[post("/execute-command", data = "<request>")]
async fn execute_confirmed_command(
    request: Json<ExecuteCommandRequest>,
    state: &State<AppState>,
) -> AppResult<Json<String>> {
    let backend = state.backend.lock().await;
    Ok(Json(
        backend
            .execute_confirmed_command(request.command.clone())
            .await?,
    ))
}

#[post("/approve-oauth", data = "<request>")]
async fn approve_oauth(
    request: Json<ApproveOauthRequest>,
    state: &State<AppState>,
) -> AppResult<Json<bool>> {
    if request.approved {
        let backend = state.backend.lock().await;
        backend.approve_oauth();
        Ok(Json(true))
    } else {
        Ok(Json(false))
    }
}

#[post("/generate-title", data = "<request>")]
async fn generate_conversation_title(
    request: Json<GenerateTitleRequest>,
    state: &State<AppState>,
) -> AppResult<Json<String>> {
    let req = request.into_inner();
    let backend = state.backend.lock().await;
    Ok(Json(
        backend
            .generate_conversation_title(req.message, req.model)
            .await?,
    ))
}

#[post("/validate-directory", data = "<request>")]
async fn validate_directory(
    request: Json<ValidateDirectoryRequest>,
    state: &State<AppState>,
) -> AppResult<Json<bool>> {
    let backend = state.backend.lock().await;
    Ok(Json(
        backend.validate_directory(request.path.clone()).await?,
    ))
}

#[post("/is-home-directory", data = "<request>")]
async fn is_home_directory(
    request: Json<IsHomeDirectoryRequest>,
    state: &State<AppState>,
) -> AppResult<Json<bool>> {
    let backend = state.backend.lock().await;
    Ok(Json(backend.is_home_directory(request.path.clone()).await?))
}

#[get("/get-home-directory")]
async fn get_home_directory(state: &State<AppState>) -> AppResult<Json<String>> {
    let backend = state.backend.lock().await;
    Ok(Json(backend.get_home_directory().await?))
}

#[post("/get-parent-directory", data = "<request>")]
async fn get_parent_directory(
    request: Json<GetParentDirectoryRequest>,
    state: &State<AppState>,
) -> AppResult<Json<Option<String>>> {
    let backend = state.backend.lock().await;
    Ok(Json(
        backend.get_parent_directory(request.path.clone()).await?,
    ))
}

#[post("/list-directory", data = "<request>")]
async fn list_directory_contents(
    request: Json<ListDirectoryRequest>,
    state: &State<AppState>,
) -> AppResult<Json<Vec<DirEntry>>> {
    let backend = state.backend.lock().await;
    Ok(Json(
        backend
            .list_directory_contents(request.path.clone())
            .await?,
    ))
}

#[post("/list-files-recursive", data = "<request>")]
async fn list_files_recursive(
    request: Json<ListFilesRecursiveRequest>,
    state: &State<AppState>,
) -> Json<Vec<DirEntry>> {
    let backend = state.backend.lock().await;
    Json(
        backend
            .list_files_recursive(request.path.clone())
            .await
            .unwrap_or_default(),
    )
}

#[get("/list-volumes")]
async fn list_volumes(state: &State<AppState>) -> AppResult<Json<Vec<DirEntry>>> {
    let backend = state.backend.lock().await;
    Ok(Json(backend.list_volumes().await?))
}

#[post("/list-gemini-commands", data = "<request>")]
async fn list_gemini_commands(
    request: Json<ListGeminiCommandsRequest>,
    state: &State<AppState>,
) -> AppResult<Json<Vec<GeminiCommand>>> {
    let backend = state.backend.lock().await;
    Ok(Json(
        backend
            .list_gemini_commands(request.working_directory.clone())
            .await?,
    ))
}

#[post("/get-git-info", data = "<request>")]
async fn get_git_info(
    request: Json<GetGitInfoRequest>,
    state: &State<AppState>,
) -> Result<Json<Option<GitInfo>>, Status> {
    let backend = state.backend.lock().await;
    match backend.get_git_info(request.path.clone()).await {
        Ok(git_info) => Ok(Json(git_info)),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[post("/read-file-content", data = "<request>")]
async fn read_file_content(
    request: Json<ReadFileContentRequest>,
    state: &State<AppState>,
) -> AppResult<Json<FileContent>> {
    let backend = state.backend.lock().await;
    Ok(Json(backend.read_file_content(request.path.clone()).await?))
}

#[post("/read-binary-file-as-base64", data = "<request>")]
async fn read_binary_file_as_base64(
    request: Json<ReadBinaryFileAsBase64Request>,
    state: &State<AppState>,
) -> AppResult<Json<String>> {
    let backend = state.backend.lock().await;
    Ok(Json(
        backend
            .read_binary_file_as_base64(request.path.clone())
            .await?,
    ))
}

#[post("/get-canonical-path", data = "<request>")]
async fn get_canonical_path(
    request: Json<CanonicalPathRequest>,
    state: &State<AppState>,
) -> AppResult<Json<String>> {
    let backend = state.backend.lock().await;
    Ok(Json(
        backend.get_canonical_path(request.path.clone()).await?,
    ))
}

#[post("/read-file-content-with-options", data = "<request>")]
async fn read_file_content_with_options(
    request: Json<ReadFileContentWithOptionsRequest>,
    state: &State<AppState>,
) -> AppResult<Json<FileContent>> {
    let backend = state.backend.lock().await;
    Ok(Json(
        backend
            .read_file_content_with_options(request.path.clone(), request.force_text)
            .await?,
    ))
}

#[post("/write-file-content", data = "<request>")]
async fn write_file_content(
    request: Json<WriteFileContentRequest>,
    state: &State<AppState>,
) -> AppResult<Json<FileContent>> {
    let backend = state.backend.lock().await;
    Ok(Json(
        backend
            .write_file_content(request.path.clone(), request.content.clone())
            .await?,
    ))
}

#[get("/ws")]
fn websocket_handler(
    ws: WebSocket,
    state: &State<AppState>,
    mut shutdown: Shutdown,
) -> Stream!['static] {
    let ws_manager = state.ws_manager.clone();
    Stream! { ws =>
        let _ = ws;
        let (tx, mut rx) = tokio_mpsc::unbounded_channel::<String>();
        let _connection_id = ws_manager.add_connection(tx.clone()).await;
        loop {
            tokio::select! {
                msg = rx.recv() => { match msg { Some(backend_msg) => yield Message::text(backend_msg), None => break } },
                _ = &mut shutdown => break
            }
        }
        ws_manager.remove_connection(&tx).await;
    }
}

#[rocket::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&args.log_level)),
        )
        .with_writer(std::io::stderr)
        .init();

    // Load original bl1nk-core config
    let config = if let Some(path) = args.config {
        Config::load(path)?
    } else {
        Config::load_default()?
    };

    // Initialize Orchestrator
    let orchestrator = Orchestrator::new(config.clone()).await?;
    let orchestrator = Arc::new(orchestrator);

    // Initialize Backend with WebSockets
    let ws_manager = WebSocketManager::new();
    let emitter = WebSocketsEventEmitter::new(ws_manager.clone());
    let backend = GeminiBackend::new(emitter);

    let app_state = AppState {
        backend: Arc::new(Mutex::new(backend)),
        ws_manager,
        orchestrator,
    };

    let _ = rocket::custom(
        rocket::Config::figment()
            .merge(("port", 1858))
            .merge(("address", "127.0.0.1")),
    )
    .manage(app_state)
    .mount("/", routes![index])
    .mount(
        "/api",
        routes![
            websocket_handler,
            check_cli_installed,
            check_cli_installed_named,
            start_session,
            start_session_with_cli,
            send_message,
            get_process_statuses,
            kill_process,
            send_tool_call_confirmation_response,
            execute_confirmed_command,
            approve_oauth,
            generate_conversation_title,
            validate_directory,
            is_home_directory,
            get_home_directory,
            get_parent_directory,
            list_directory_contents,
            list_files_recursive,
            list_volumes,
            list_gemini_commands,
            get_git_info,
            get_recent_chats,
            search_chats,
            list_projects,
            list_enriched_projects,
            get_enriched_project_http,
            get_project_discussions,
            get_detailed_conversation,
            export_conversation_history,
            read_file_content,
            read_binary_file_as_base64,
            get_canonical_path,
            read_file_content_with_options,
            write_file_content,
        ],
    )
    .launch()
    .await?;

    Ok(())
}
