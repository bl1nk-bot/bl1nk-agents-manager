mod concurrency;
mod constants;
mod results;
mod state;
mod task_state;
mod spawner;
mod types;

use concurrency::ConcurrencyManager;
use tokio::time::sleep;
use state::TaskStateManager;
use spawner::{create_task, resume_task, start_task as spawn_task, QueueItem, SpawnerContext};
use types::{
    BackgroundTask, BackgroundTaskConfig, BackgroundTaskProgress, BackgroundTaskStatus, LaunchInput,
    ModelRef, ResumeInput, TmuxConfig, Todo,
};

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use tokio::task::JoinHandle;

pub use concurrency::ConcurrencyManager as BackgroundConcurrencyManager;
pub use constants::*;
#[allow(unused_imports)]
pub use results::*;
#[allow(unused_imports)]
pub use state::*;
#[allow(unused_imports)]
pub use types::*;

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: String,
    pub directory: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStatus {
    #[serde(rename = "type")]
    pub status_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageInfo {
    pub role: Option<String>,
    pub agent: Option<String>,
    pub model: Option<ModelRef>,
    pub model_id: Option<String>,
    pub provider_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagePart {
    #[serde(rename = "type")]
    pub part_type: String,
    pub text: Option<String>,
    pub tool: Option<String>,
    pub name: Option<String>,
    pub content: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub info: Option<MessageInfo>,
    pub parts: Option<Vec<MessagePart>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptRequest {
    pub agent: Option<String>,
    pub model: Option<ModelRef>,
    pub no_reply: bool,
    pub system: Option<String>,
    pub parts: Vec<MessagePart>,
}

pub trait BackgroundClient: Send + Sync {
    fn session_get<'a>(&'a self, id: &'a str) -> BoxFuture<'a, Option<SessionInfo>>;
    fn session_create<'a>(
        &'a self,
        parent_id: &'a str,
        title: &'a str,
        directory: &'a str,
    ) -> BoxFuture<'a, Option<SessionInfo>>;
    fn session_prompt<'a>(&'a self, id: &'a str, request: PromptRequest) -> BoxFuture<'a, bool>;
    fn session_abort<'a>(&'a self, id: &'a str) -> BoxFuture<'a, ()>;
    fn session_messages<'a>(&'a self, id: &'a str) -> BoxFuture<'a, Vec<Message>>;
    fn session_status<'a>(&'a self) -> BoxFuture<'a, HashMap<String, SessionStatus>>;
    fn session_todo<'a>(&'a self, id: &'a str) -> BoxFuture<'a, Vec<Todo>>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    pub properties: serde_json::Value,
}

#[derive(Debug, Clone)]
struct StartTaskError {
    task_id: String,
    message: String,
}

#[derive(Debug, Clone)]
pub struct SubagentSessionCreatedEvent {
    pub session_id: String,
    pub parent_id: String,
    pub title: String,
}

pub type OnSubagentSessionCreated = Arc<dyn Fn(SubagentSessionCreatedEvent) -> BoxFuture<'static, ()> + Send + Sync>;

pub struct BackgroundManager<C: BackgroundClient + 'static> {
    client: Arc<C>,
    directory: String,
    config: Option<BackgroundTaskConfig>,
    tmux_enabled: bool,
    on_subagent_session_created: Option<OnSubagentSessionCreated>,
    on_shutdown: Option<Arc<dyn Fn() + Send + Sync>>,

    tasks: Arc<Mutex<HashMap<String, BackgroundTask>>>,
    notifications: Arc<Mutex<HashMap<String, Vec<BackgroundTask>>>>,
    pending_by_parent: Arc<Mutex<HashMap<String, HashSet<String>>>>,
    queues_by_key: Arc<Mutex<HashMap<String, VecDeque<QueueItem>>>>,
    processing_keys: Arc<Mutex<HashSet<String>>>,
    completion_timers: Arc<Mutex<HashMap<String, JoinHandle<()>>>>,
    idle_deferral_timers: Arc<Mutex<HashMap<String, JoinHandle<()>>>>,
    polling_task: Arc<Mutex<Option<JoinHandle<()>>>>,
    concurrency_manager: ConcurrencyManager,
    shutdown_triggered: Arc<Mutex<bool>>,
}

impl<C: BackgroundClient + 'static> Clone for BackgroundManager<C> {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            directory: self.directory.clone(),
            config: self.config.clone(),
            tmux_enabled: self.tmux_enabled,
            on_subagent_session_created: self.on_subagent_session_created.clone(),
            on_shutdown: self.on_shutdown.clone(),
            tasks: self.tasks.clone(),
            notifications: self.notifications.clone(),
            pending_by_parent: self.pending_by_parent.clone(),
            queues_by_key: self.queues_by_key.clone(),
            processing_keys: self.processing_keys.clone(),
            completion_timers: self.completion_timers.clone(),
            idle_deferral_timers: self.idle_deferral_timers.clone(),
            polling_task: self.polling_task.clone(),
            concurrency_manager: self.concurrency_manager.clone(),
            shutdown_triggered: self.shutdown_triggered.clone(),
        }
    }
}

impl<C: BackgroundClient + 'static> BackgroundManager<C> {
    pub fn new(
        client: Arc<C>,
        directory: String,
        config: Option<BackgroundTaskConfig>,
        tmux_config: Option<TmuxConfig>,
        on_subagent_session_created: Option<OnSubagentSessionCreated>,
        on_shutdown: Option<Arc<dyn Fn() + Send + Sync>>,
    ) -> Self {
        let limit = config.as_ref().and_then(|c| c.max_concurrency_per_key);
        Self {
            client,
            directory,
            config,
            tmux_enabled: tmux_config.map(|c| c.enabled).unwrap_or(false),
            on_subagent_session_created,
            on_shutdown,
            tasks: Arc::new(Mutex::new(HashMap::new())),
            notifications: Arc::new(Mutex::new(HashMap::new())),
            pending_by_parent: Arc::new(Mutex::new(HashMap::new())),
            queues_by_key: Arc::new(Mutex::new(HashMap::new())),
            processing_keys: Arc::new(Mutex::new(HashSet::new())),
            completion_timers: Arc::new(Mutex::new(HashMap::new())),
            idle_deferral_timers: Arc::new(Mutex::new(HashMap::new())),
            polling_task: Arc::new(Mutex::new(None)),
            concurrency_manager: ConcurrencyManager::new(limit),
            shutdown_triggered: Arc::new(Mutex::new(false)),
        }
    }

    fn state(&self) -> TaskStateManager {
        TaskStateManager::from_parts(
            self.tasks.clone(),
            self.notifications.clone(),
            self.pending_by_parent.clone(),
            self.queues_by_key.clone(),
            self.processing_keys.clone(),
            self.completion_timers.clone(),
        )
    }

    pub async fn launch(&self, input: LaunchInput) -> Result<BackgroundTask, String> {
        if input.agent.trim().is_empty() {
            return Err("Agent parameter is required".to_string());
        }

        let task = create_task(&input);

        {
            let mut guard = self.tasks.lock().map_err(|_| "tasks lock failed")?;
            guard.insert(task.id.clone(), task.clone());
        }

        if let Some(parent_id) = &task.parent_session_id {
            let mut pending = self
                .pending_by_parent
                .lock()
                .map_err(|_| "pending lock failed")?;
            pending
                .entry(parent_id.clone())
                .or_insert_with(HashSet::new)
                .insert(task.id.clone());
        }

        let key = self.get_concurrency_key_from_input(&input);
        {
            let mut queues = self
                .queues_by_key
                .lock()
                .map_err(|_| "queues lock failed")?;
            let queue = queues.entry(key.clone()).or_insert_with(VecDeque::new);
            queue.push_back(QueueItem {
                task: task.clone(),
                input,
            });
        }

        let manager = self.clone();
        tokio::spawn(async move {
            manager.process_key(key).await;
        });

        Ok(task)
    }

    async fn process_key(&self, key: String) {
        {
            let mut processing = match self.processing_keys.lock() {
                Ok(guard) => guard,
                Err(_) => return,
            };
            if processing.contains(&key) {
                return;
            }
            processing.insert(key.clone());
        }

        loop {
            let item = {
                let mut queues = match self.queues_by_key.lock() {
                    Ok(guard) => guard,
                    Err(_) => break,
                };
                if let Some(queue) = queues.get_mut(&key) {
                    let popped = queue.pop_front();
                    if popped.is_none() && queue.is_empty() {
                        queues.remove(&key);
                    }
                    popped
                } else {
                    None
                }
            };

            let item = match item {
                Some(i) => i,
                None => break,
            };

            self.concurrency_manager.acquire(&key).await;

            let cancelled = {
                let mut tasks = match self.tasks.lock() {
                    Ok(guard) => guard,
                    Err(_) => break,
                };
                if let Some(t) = tasks.get_mut(&item.task.id) {
                    if t.status == BackgroundTaskStatus::Cancelled {
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            };
            if cancelled {
                self.concurrency_manager.release(&key);
                continue;
            }

            if let Err(err) = self.start_task(item).await {
                let mut notify_task: Option<BackgroundTask> = None;
                {
                    let mut tasks = self.tasks.lock().unwrap();
                    if let Some(existing) = tasks.get_mut(&err.task_id) {
                        existing.status = BackgroundTaskStatus::Error;
                        existing.error = Some(err.message);
                        existing.completed_at = Some(SystemTime::now());
                        if let Some(key) = existing.concurrency_key.take() {
                            self.concurrency_manager.release(&key);
                        }
                        notify_task = Some(existing.clone());
                    }
                }

                if let Some(task) = notify_task {
                    self.mark_for_notification(&task);
                    let ctx = ResultHandlerContext {
                        client: self.client.clone(),
                        concurrency_manager: self.concurrency_manager.clone(),
                        state: self.state(),
                    };
                    let _ = results::notify_parent_session(&task, &ctx).await;
                }
                self.concurrency_manager.release(&key);
            }
        }

        let mut processing = match self.processing_keys.lock() {
            Ok(guard) => guard,
            Err(_) => return,
        };
        processing.remove(&key);
    }

    async fn start_task(&self, item: QueueItem) -> Result<(), StartTaskError> {
        let ctx = SpawnerContext {
            client: self.client.clone(),
            directory: self.directory.clone(),
            tmux_enabled: self.tmux_enabled,
            on_subagent_session_created: self.on_subagent_session_created.clone(),
        };
        let task = spawn_task(item.clone(), &ctx)
            .await
            .map_err(|message| StartTaskError {
                task_id: item.task.id.clone(),
                message,
            })?;
        let mut tasks = self.tasks.lock().map_err(|_| StartTaskError {
            task_id: item.task.id.clone(),
            message: "tasks lock failed".to_string(),
        })?;
        tasks.insert(task.id.clone(), task);
        self.start_polling();
        Ok(())
    }

    pub fn get_task(&self, id: &str) -> Option<BackgroundTask> {
        self.tasks
            .lock()
            .ok()
            .and_then(|guard| guard.get(id).cloned())
    }

    pub fn get_tasks_by_parent_session(&self, session_id: &str) -> Vec<BackgroundTask> {
        let guard = match self.tasks.lock() {
            Ok(g) => g,
            Err(_) => return Vec::new(),
        };
        guard
            .values()
            .filter(|t| t.parent_session_id.as_deref() == Some(session_id))
            .cloned()
            .collect()
    }

    pub fn get_all_descendant_tasks(&self, session_id: &str) -> Vec<BackgroundTask> {
        let mut result = Vec::new();
        let direct = self.get_tasks_by_parent_session(session_id);
        for child in direct {
            result.push(child.clone());
            if let Some(session_id) = &child.session_id {
                result.extend(self.get_all_descendant_tasks(session_id));
            }
        }
        result
    }

    pub fn find_by_session(&self, session_id: &str) -> Option<BackgroundTask> {
        let guard = self.tasks.lock().ok()?;
        guard.values().find(|t| t.session_id.as_deref() == Some(session_id)).cloned()
    }

    pub async fn track_task(&self, task: BackgroundTask) -> BackgroundTask {
        {
            let mut tasks = self.tasks.lock().expect("tasks lock failed");
            tasks.insert(task.id.clone(), task.clone());
        }

        if let Some(parent) = &task.parent_session_id {
            let mut pending = self.pending_by_parent.lock().expect("pending lock failed");
            pending
                .entry(parent.clone())
                .or_insert_with(HashSet::new)
                .insert(task.id.clone());
        }

        self.start_polling();
        task
    }

    pub async fn resume(&self, input: ResumeInput) -> Result<BackgroundTask, String> {
        let mut task = self
            .find_by_session(&input.session_id)
            .ok_or_else(|| format!("Task not found for session: {}", input.session_id))?;

        if task.status == BackgroundTaskStatus::Running {
            return Ok(task);
        }

        task = resume_task(
            task,
            &input,
            self.client.clone(),
            &self.concurrency_manager,
        )
        .await
        .map_err(|e| e)?;

        let mut tasks = self.tasks.lock().map_err(|_| "tasks lock failed")?;
        tasks.insert(task.id.clone(), task.clone());
        self.start_polling();
        Ok(task)
    }

    pub async fn cancel_task(
        &self,
        task_id: &str,
        reason: Option<String>,
        abort_session: bool,
        skip_notification: bool,
    ) -> bool {
        let mut task = match self.get_task(task_id) {
            Some(t) => t,
            None => return false,
        };

        if task.status != BackgroundTaskStatus::Running
            && task.status != BackgroundTaskStatus::Pending
        {
            return false;
        }

        if task.status == BackgroundTaskStatus::Pending {
            let key = task
                .model
                .as_ref()
                .map(|m| format!("{}/{}", m.provider_id, m.model_id))
                .unwrap_or_else(|| task.agent.clone());
            let mut queues = self.queues_by_key.lock().ok();
            if let Some(ref mut queues) = queues {
                if let Some(queue) = queues.get_mut(&key) {
                    queue.retain(|item| item.task.id != task_id);
                    if queue.is_empty() {
                        queues.remove(&key);
                    }
                }
            }
        }

        task.status = BackgroundTaskStatus::Cancelled;
        task.completed_at = Some(SystemTime::now());
        if let Some(r) = reason {
            task.error = Some(r);
        }

        if let Some(key) = task.concurrency_key.take() {
            self.concurrency_manager.release(&key);
        }

        if abort_session {
            if let Some(session_id) = &task.session_id {
                self.client.session_abort(session_id).await;
            }
        }

        {
            let mut tasks = self.tasks.lock().expect("tasks lock failed");
            tasks.insert(task.id.clone(), task.clone());
        }

        if !skip_notification {
            self.mark_for_notification(&task);
            let ctx = ResultHandlerContext {
                client: self.client.clone(),
                concurrency_manager: self.concurrency_manager.clone(),
                state: self.state(),
            };
            let _ = results::notify_parent_session(&task, &ctx).await;
        }

        true
    }

    pub fn cancel_pending_task(&self, task_id: &str) -> bool {
        let task = self.get_task(task_id);
        if task
            .as_ref()
            .map(|t| t.status.clone())
            != Some(BackgroundTaskStatus::Pending)
        {
            return false;
        }
        let manager = Arc::new(self.clone());
        let task_id = task_id.to_string();
        tokio::spawn(async move {
            let _ = manager
                .cancel_task(&task_id, Some("cancelled".to_string()), false, true)
                .await;
        });
        true
    }

    pub fn handle_event(&self, event: BackgroundEvent) {
        if event.event_type == "message.part.updated" {
            let session_id = event
                .properties
                .get("sessionID")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            if let Some(session_id) = session_id {
                if let Some(mut task) = self.find_by_session(&session_id) {
                    if let Some(progress) = task.progress.as_mut() {
                        let tool = event
                            .properties
                            .get("tool")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        if tool.is_some() {
                            progress.tool_calls += 1;
                            progress.last_tool = tool;
                            progress.last_update = SystemTime::now();
                        }
                    }
                    let mut tasks = self.tasks.lock().unwrap();
                    tasks.insert(task.id.clone(), task);
                }
            }
        }
    }

    fn start_polling(&self) {
        let mut guard = self.polling_task.lock().expect("polling lock failed");
        if guard.is_some() {
            return;
        }

        let manager = Arc::new(self.clone());
        let handle = tokio::spawn(async move {
            loop {
                manager.poll_running_tasks().await;
                sleep(Duration::from_millis(constants::POLLING_INTERVAL_MS)).await;
                if !manager.has_running_tasks() {
                    break;
                }
            }
        });
        *guard = Some(handle);
    }

    fn stop_polling(&self) {
        let mut guard = self.polling_task.lock().expect("polling lock failed");
        if let Some(handle) = guard.take() {
            handle.abort();
        }
    }

    fn mark_for_notification(&self, task: &BackgroundTask) {
        self.state().mark_for_notification(task);
    }

    fn has_running_tasks(&self) -> bool {
        let guard = match self.tasks.lock() {
            Ok(g) => g,
            Err(_) => return false,
        };
        guard
            .values()
            .any(|t| t.status == BackgroundTaskStatus::Running)
    }

    fn prune_stale_tasks_and_notifications(&self) {
        let now = SystemTime::now();
        let mut tasks = match self.tasks.lock() {
            Ok(g) => g,
            Err(_) => return,
        };
        let mut to_remove = Vec::new();
        for (id, task) in tasks.iter_mut() {
            let timestamp = task
                .queued_at
                .or(task.started_at)
                .unwrap_or_else(SystemTime::now);
            let age = now
                .duration_since(timestamp)
                .unwrap_or_else(|_| Duration::from_secs(0))
                .as_millis() as u64;
            if age > constants::TASK_TTL_MS {
                task.status = BackgroundTaskStatus::Error;
                task.error = Some(if task.status == BackgroundTaskStatus::Pending {
                    "Task timed out while queued (30 minutes)".to_string()
                } else {
                    "Task timed out after 30 minutes".to_string()
                });
                task.completed_at = Some(SystemTime::now());
                if let Some(key) = task.concurrency_key.take() {
                    self.concurrency_manager.release(&key);
                }
                to_remove.push(id.clone());
            }
        }
        for id in to_remove {
            tasks.remove(&id);
        }
    }

    async fn check_and_interrupt_stale_tasks(&self) {
        let stale_timeout_ms = self
            .config
            .as_ref()
            .and_then(|c| c.stale_timeout_ms)
            .unwrap_or(constants::DEFAULT_STALE_TIMEOUT_MS);
        let now = SystemTime::now();
        let tasks = self.tasks.lock().unwrap().values().cloned().collect::<Vec<_>>();

        for mut task in tasks {
            if task.status != BackgroundTaskStatus::Running {
                continue;
            }
            let last_update = match task.progress.as_ref().map(|p| p.last_update) {
                Some(t) => t,
                None => continue,
            };
            let started_at = match task.started_at {
                Some(t) => t,
                None => continue,
            };
            let runtime = now
                .duration_since(started_at)
                .unwrap_or_else(|_| Duration::from_secs(0))
                .as_millis() as u64;
            if runtime < constants::MIN_RUNTIME_BEFORE_STALE_MS {
                continue;
            }
            let time_since_last = now
                .duration_since(last_update)
                .unwrap_or_else(|_| Duration::from_secs(0))
                .as_millis() as u64;
            if time_since_last <= stale_timeout_ms {
                continue;
            }

            task.status = BackgroundTaskStatus::Cancelled;
            task.error = Some(format!(
                "Stale timeout (no activity for {}min)",
                time_since_last / 60000
            ));
            task.completed_at = Some(SystemTime::now());

            if let Some(key) = task.concurrency_key.take() {
                self.concurrency_manager.release(&key);
            }

            if let Some(session_id) = &task.session_id {
                self.client.session_abort(session_id).await;
            }

            {
                let mut tasks_map = self.tasks.lock().unwrap();
                tasks_map.insert(task.id.clone(), task.clone());
            }
            let ctx = ResultHandlerContext {
                client: self.client.clone(),
                concurrency_manager: self.concurrency_manager.clone(),
                state: self.state(),
            };
            let _ = results::notify_parent_session(&task, &ctx).await;
        }
    }

    async fn poll_running_tasks(&self) {
        self.prune_stale_tasks_and_notifications();
        self.check_and_interrupt_stale_tasks().await;

        let statuses = self.client.session_status().await;

        let tasks_snapshot = self.tasks.lock().unwrap().values().cloned().collect::<Vec<_>>();
        for mut task in tasks_snapshot {
            if task.status != BackgroundTaskStatus::Running {
                continue;
            }
            let session_id = match task.session_id.clone() {
                Some(id) => id,
                None => continue,
            };

            if let Some(status) = statuses.get(&session_id) {
                if status.status_type == "idle" {
                    if !results::validate_session_has_output(self.client.clone(), &session_id).await {
                        continue;
                    }
                    if results::check_session_todos(self.client.clone(), &session_id).await {
                        continue;
                    }
                    let ctx = ResultHandlerContext {
                        client: self.client.clone(),
                        concurrency_manager: self.concurrency_manager.clone(),
                        state: self.state(),
                    };
                    let _ = results::try_complete_task(&mut task, "polling (idle status)", &ctx).await;
                    continue;
                }
            }

            let messages = self.client.session_messages(&session_id).await;
            let assistant_msgs = messages
                .iter()
                .filter(|m| {
                    m.info
                        .as_ref()
                        .and_then(|i| i.role.as_ref())
                        .map(|r| r == "assistant")
                        .unwrap_or(false)
                })
                .collect::<Vec<_>>();

            let mut tool_calls = 0;
            let mut last_tool = None;
            let mut last_message = None;

            for msg in assistant_msgs {
                let empty_parts: Vec<MessagePart> = Vec::new();
                let parts = msg.parts.as_ref().unwrap_or(&empty_parts);
                for part in parts {
                    if part.part_type == "tool_use" || part.tool.is_some() {
                        tool_calls += 1;
                        last_tool = part.tool.clone().or(part.name.clone());
                    }
                    if part.part_type == "text" {
                        last_message = part.text.clone();
                    }
                }
            }

            if task.progress.is_none() {
                task.progress = Some(BackgroundTaskProgress {
                    tool_calls: 0,
                    last_update: SystemTime::now(),
                    last_tool: None,
                    last_message: None,
                    last_message_at: None,
                });
            }
            if let Some(progress) = task.progress.as_mut() {
                progress.tool_calls = tool_calls;
                progress.last_tool = last_tool;
                progress.last_update = SystemTime::now();
                if last_message.is_some() {
                    progress.last_message = last_message;
                    progress.last_message_at = Some(SystemTime::now());
                }
            }

            let current_msg_count = messages.len();
            let started_at = match task.started_at {
                Some(t) => t,
                None => continue,
            };
            let elapsed_ms = SystemTime::now()
                .duration_since(started_at)
                .unwrap_or_else(|_| Duration::from_secs(0))
                .as_millis() as u64;

            if elapsed_ms >= constants::MIN_STABILITY_TIME_MS {
                if task.last_msg_count == Some(current_msg_count) {
                    task.stable_polls = Some(task.stable_polls.unwrap_or(0) + 1);
                    if task.stable_polls.unwrap_or(0) >= 3 {
                        if !results::validate_session_has_output(self.client.clone(), &session_id).await {
                            continue;
                        }
                        if !results::check_session_todos(self.client.clone(), &session_id).await {
                            let ctx = ResultHandlerContext {
                                client: self.client.clone(),
                                concurrency_manager: self.concurrency_manager.clone(),
                                state: self.state(),
                            };
                            let _ = results::try_complete_task(&mut task, "stability detection", &ctx).await;
                        }
                    }
                } else {
                    task.stable_polls = Some(0);
                }
            }
            task.last_msg_count = Some(current_msg_count);

            let mut tasks = self.tasks.lock().unwrap();
            tasks.insert(task.id.clone(), task);
        }

        if !self.has_running_tasks() {
            self.stop_polling();
        }
    }

    fn get_concurrency_key_from_input(&self, input: &LaunchInput) -> String {
        if let Some(model) = &input.model {
            format!("{}/{}", model.provider_id, model.model_id)
        } else {
            input.agent.clone()
        }
    }

    pub fn shutdown(&self) {
        let mut guard = self.shutdown_triggered.lock().unwrap();
        if *guard {
            return;
        }
        *guard = true;

        self.stop_polling();
        {
            let tasks = self.tasks.lock().unwrap();
            for task in tasks.values() {
                if task.status == BackgroundTaskStatus::Running {
                    if let Some(session_id) = &task.session_id {
                        let client = self.client.clone();
                        let session_id = session_id.clone();
                        tokio::spawn(async move {
                            client.session_abort(&session_id).await;
                        });
                    }
                }
            }
        }

        if let Some(callback) = &self.on_shutdown {
            callback();
        }

        self.concurrency_manager.clear();

        self.tasks.lock().unwrap().clear();
        self.notifications.lock().unwrap().clear();
        self.pending_by_parent.lock().unwrap().clear();
        self.queues_by_key.lock().unwrap().clear();
        self.processing_keys.lock().unwrap().clear();

        let mut timers = self.completion_timers.lock().unwrap();
        for (_, handle) in timers.drain() {
            handle.abort();
        }

        let mut idle_timers = self.idle_deferral_timers.lock().unwrap();
        for (_, handle) in idle_timers.drain() {
            handle.abort();
        }
    }
}
