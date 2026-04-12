use super::{BackgroundClient, BackgroundTask, BackgroundTaskStatus, PromptRequest};
use super::{TaskStateManager, TASK_CLEANUP_DELAY_MS};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};

pub struct ResultHandlerContext<C: BackgroundClient + 'static> {
    pub client: Arc<C>,
    pub concurrency_manager: super::ConcurrencyManager,
    pub state: TaskStateManager,
}

pub const MESSAGE_STORAGE_ENV: &str = "MESSAGE_STORAGE";

pub async fn check_session_todos<C: BackgroundClient + 'static>(
    client: Arc<C>,
    session_id: &str,
) -> bool {
    let todos = client.session_todo(session_id).await;
    if todos.is_empty() {
        return false;
    }
    let incomplete = todos
        .into_iter()
        .filter(|t| t.status != "completed" && t.status != "cancelled")
        .count();
    incomplete > 0
}

pub async fn validate_session_has_output<C: BackgroundClient + 'static>(
    client: Arc<C>,
    session_id: &str,
) -> bool {
    let messages = client.session_messages(session_id).await;

    let has_assistant_or_tool = messages.iter().any(|m| {
        m.info
            .as_ref()
            .and_then(|i| i.role.as_ref())
            .map(|r| r == "assistant" || r == "tool")
            .unwrap_or(false)
    });

    if !has_assistant_or_tool {
        return false;
    }

    let has_content = messages.iter().any(|m| {
        m.info
            .as_ref()
            .and_then(|i| i.role.as_ref())
            .map(|r| r == "assistant" || r == "tool")
            .unwrap_or(false)
            && m
                .parts
                .as_ref()
                .unwrap_or(&vec![])
                .iter()
                .any(|p| {
                    (p.part_type == "text"
                        && p.text.as_ref().map(|t| !t.trim().is_empty()).unwrap_or(false))
                        || (p.part_type == "reasoning"
                            && p.text.as_ref().map(|t| !t.trim().is_empty()).unwrap_or(false))
                        || p.part_type == "tool"
                        || (p.part_type == "tool_result"
                            && p.content.as_ref().map(|c| !c.is_null()).unwrap_or(false))
                })
    });

    has_content
}

pub fn format_duration(start: SystemTime, end: Option<SystemTime>) -> String {
    let end = end.unwrap_or_else(SystemTime::now);
    let duration = end
        .duration_since(start)
        .unwrap_or_else(|_| Duration::from_secs(0));
    let seconds = duration.as_secs();
    let minutes = seconds / 60;
    let hours = minutes / 60;
    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes % 60, seconds % 60)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds % 60)
    } else {
        format!("{}s", seconds)
    }
}

pub fn get_message_dir(session_id: &str) -> Option<PathBuf> {
    let base = std::env::var(MESSAGE_STORAGE_ENV).ok()?;
    let base = PathBuf::from(base);
    let direct = base.join(session_id);
    if direct.exists() {
        return Some(direct);
    }
    if let Ok(entries) = std::fs::read_dir(&base) {
        for entry in entries.flatten() {
            let session_path = entry.path().join(session_id);
            if session_path.exists() {
                return Some(session_path);
            }
        }
    }
    None
}

pub async fn try_complete_task<C: BackgroundClient + 'static>(
    task: &mut BackgroundTask,
    source: &str,
    ctx: &ResultHandlerContext<C>,
) -> bool {
    if task.status != BackgroundTaskStatus::Running {
        return false;
    }

    task.status = BackgroundTaskStatus::Completed;
    task.completed_at = Some(SystemTime::now());

    if let Some(key) = task.concurrency_key.take() {
        ctx.concurrency_manager.release(&key);
    }

    ctx.state.mark_for_notification(task);
    if let Some(parent_id) = task.parent_session_id.as_deref() {
        ctx.state.update_pending(parent_id, &task.id);
    }

    if let Some(session_id) = &task.session_id {
        ctx.client.session_abort(session_id).await;
    }

    let _ = source;
    let _ = notify_parent_session(task, ctx).await;
    let tasks_map = ctx.state.tasks_map();
    let mut tasks = tasks_map.lock().unwrap();
    tasks.insert(task.id.clone(), task.clone());

    true
}

pub async fn notify_parent_session<C: BackgroundClient + 'static>(
    task: &BackgroundTask,
    ctx: &ResultHandlerContext<C>,
) -> Result<(), String> {
    let parent_id = task
        .parent_session_id
        .as_ref()
        .ok_or("Missing parent session")?;

    let duration = format_duration(task.started_at.unwrap_or(SystemTime::now()), task.completed_at);

    let remaining = ctx.state.pending_set_size(parent_id).unwrap_or(0);
    let all_complete = remaining == 0;

    let status_text = match task.status {
        BackgroundTaskStatus::Completed => "COMPLETED",
        BackgroundTaskStatus::Cancelled => "CANCELLED",
        BackgroundTaskStatus::Error => "FAILED",
        _ => "UPDATED",
    };

    let error_info = task
        .error
        .as_ref()
        .map(|e| format!("\n**Error:** {}", e))
        .unwrap_or_default();

    let notification = if all_complete {
        let completed_tasks = ctx
            .state
            .tasks_for_parent(parent_id)
            .into_iter()
            .filter(|t| t.status != BackgroundTaskStatus::Running && t.status != BackgroundTaskStatus::Pending)
            .map(|t| format!("- `{}`: {}", t.id, t.description))
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            "<system-reminder>\n[ALL BACKGROUND TASKS COMPLETE]\n\n**Completed:**\n{}\n\nUse `background_output(task_id=\"<id>\")` to retrieve each result.\n</system-reminder>",
            if completed_tasks.is_empty() {
                format!("- `{}`: {}", task.id, task.description)
            } else {
                completed_tasks
            }
        )
    } else {
        let agent_info = task
            .category
            .as_ref()
            .map(|c| format!("{} ({})", task.agent, c))
            .unwrap_or_else(|| task.agent.clone());
        format!(
            "<system-reminder>\n[BACKGROUND TASK {}]\n**ID:** `{}`\n**Description:** {}\n**Agent:** {}\n**Duration:** {}{}\n\n**{} task{} still in progress.** You WILL be notified when ALL complete.\nDo NOT poll - continue productive work.\n\nUse `background_output(task_id=\"{}\")` to retrieve this result when ready.\n</system-reminder>",
            status_text,
            task.id,
            task.description,
            agent_info,
            duration,
            error_info,
            remaining,
            if remaining == 1 { "" } else { "s" },
            task.id
        )
    };

    let prompt = PromptRequest {
        agent: task.parent_agent.clone(),
        model: task.parent_model.clone(),
        no_reply: !all_complete,
        system: None,
        parts: vec![super::MessagePart {
            part_type: "text".to_string(),
            text: Some(notification),
            tool: None,
            name: None,
            content: None,
        }],
    };

    let ok = ctx.client.session_prompt(parent_id, prompt).await;
    if !ok {
        return Err("Failed to send notification".to_string());
    }

    if all_complete {
        let manager = ctx.state.clone();
        let task_id = task.id.clone();
        let manager_for_task = manager.clone();
        let task_id_for_task = task_id.clone();
        let handle: JoinHandle<()> = tokio::spawn(async move {
            sleep(Duration::from_millis(TASK_CLEANUP_DELAY_MS)).await;
            let tasks_map = manager_for_task.tasks_map();
            let mut tasks = tasks_map.lock().unwrap();
            tasks.remove(&task_id_for_task);
        });
        manager.set_completion_timer(&task_id, handle);
    }

    Ok(())
}
