use super::{
    BackgroundClient, BackgroundTask, BackgroundTaskProgress, BackgroundTaskStatus, LaunchInput,
    ResumeInput, TMUX_CALLBACK_DELAY_MS,
};
use super::{ConcurrencyManager, ModelRef, OnSubagentSessionCreated, PromptRequest};
use std::sync::Arc;
use std::time::SystemTime;
use tokio::time::{sleep, Duration};
use uuid::Uuid;

#[derive(Clone)]
pub struct QueueItem {
    pub task: BackgroundTask,
    pub input: LaunchInput,
}

pub struct SpawnerContext<C: BackgroundClient + 'static> {
    pub client: Arc<C>,
    pub directory: String,
    pub tmux_enabled: bool,
    pub on_subagent_session_created: Option<OnSubagentSessionCreated>,
}

pub fn create_task(input: &LaunchInput) -> BackgroundTask {
    BackgroundTask {
        id: format!("bg_{}", Uuid::new_v4().simple().to_string()[..8].to_string()),
        status: BackgroundTaskStatus::Pending,
        queued_at: Some(SystemTime::now()),
        started_at: None,
        completed_at: None,
        description: input.description.clone(),
        prompt: input.prompt.clone(),
        agent: input.agent.clone(),
        result: None,
        parent_session_id: Some(input.parent_session_id.clone()),
        parent_message_id: input.parent_message_id.clone(),
        parent_model: input.parent_model.clone(),
        parent_agent: input.parent_agent.clone(),
        is_unstable_agent: input.is_unstable_agent,
        model: input.model.clone(),
        category: input.category.clone(),
        progress: None,
        concurrency_key: None,
        concurrency_group: None,
        error: None,
        last_msg_count: None,
        stable_polls: None,
        session_id: None,
    }
}

pub async fn start_task<C: BackgroundClient + 'static>(
    mut item: QueueItem,
    ctx: &SpawnerContext<C>,
) -> Result<BackgroundTask, String> {
    let input = &item.input;

    let concurrency_key = if let Some(model) = &input.model {
        format!("{}/{}", model.provider_id, model.model_id)
    } else {
        input.agent.clone()
    };

    let parent_session = ctx.client.session_get(&input.parent_session_id).await;
    let parent_directory = parent_session
        .as_ref()
        .and_then(|s| s.directory.clone())
        .unwrap_or_else(|| ctx.directory.clone());

    let title = format!("Background: {}", input.description);
    let session = ctx
        .client
        .session_create(&input.parent_session_id, &title, &parent_directory)
        .await
        .ok_or_else(|| "Failed to create background session".to_string())?;

    let session_id = session.id.clone();

    if let Some(callback) = &ctx.on_subagent_session_created {
        if ctx.tmux_enabled && is_inside_tmux() {
            let cb = callback.clone();
            let parent_id = input.parent_session_id.clone();
            let title = input.description.clone();
            let session_id_clone = session_id.clone();
            tokio::spawn(async move {
                cb(super::SubagentSessionCreatedEvent {
                    session_id: session_id_clone,
                    parent_id,
                    title,
                })
                .await;
            });
            sleep(Duration::from_millis(TMUX_CALLBACK_DELAY_MS)).await;
        }
    }

    item.task.status = BackgroundTaskStatus::Running;
    item.task.started_at = Some(SystemTime::now());
    item.task.session_id = Some(session_id.clone());
    item.task.progress = Some(BackgroundTaskProgress {
        tool_calls: 0,
        last_update: SystemTime::now(),
        last_tool: None,
        last_message: None,
        last_message_at: None,
    });
    item.task.concurrency_key = Some(concurrency_key.clone());
    item.task.concurrency_group = Some(concurrency_key.clone());

    let launch_model = input.model.clone().map(|m| ModelRef {
        provider_id: m.provider_id,
        model_id: m.model_id,
        variant: None,
    });
    let launch_variant = input.model.clone().and_then(|m| m.variant);

    let request = PromptRequest {
        agent: Some(input.agent.clone()),
        model: launch_model.map(|mut m| {
            if let Some(variant) = launch_variant {
                m.variant = Some(variant);
            }
            m
        }),
        no_reply: false,
        system: input.skill_content.clone(),
        parts: vec![super::MessagePart {
            part_type: "text".to_string(),
            text: Some(input.prompt.clone()),
            tool: None,
            name: None,
            content: None,
        }],
    };

    let ok = ctx.client.session_prompt(&session_id, request).await;
    if !ok {
        return Err("Failed to prompt background session".to_string());
    }

    Ok(item.task)
}

pub async fn resume_task<C: BackgroundClient + 'static>(
    mut task: BackgroundTask,
    input: &ResumeInput,
    client: Arc<C>,
    concurrency_manager: &ConcurrencyManager,
) -> Result<BackgroundTask, String> {
    let session_id = task
        .session_id
        .clone()
        .ok_or_else(|| format!("Task has no sessionID: {}", task.id))?;

    if task.status == BackgroundTaskStatus::Running {
        return Ok(task);
    }

    let concurrency_key = task
        .concurrency_group
        .clone()
        .unwrap_or_else(|| task.agent.clone());
    concurrency_manager.acquire(&concurrency_key).await;
    task.concurrency_key = Some(concurrency_key.clone());
    task.concurrency_group = Some(concurrency_key);

    task.status = BackgroundTaskStatus::Running;
    task.completed_at = None;
    task.error = None;
    task.parent_session_id = Some(input.parent_session_id.clone());
    task.parent_message_id = input.parent_message_id.clone();
    task.parent_model = input.parent_model.clone();
    task.parent_agent = input.parent_agent.clone();
    task.started_at = Some(SystemTime::now());
    task.progress = Some(BackgroundTaskProgress {
        tool_calls: task.progress.as_ref().map(|p| p.tool_calls).unwrap_or(0),
        last_update: SystemTime::now(),
        last_tool: None,
        last_message: None,
        last_message_at: None,
    });

    let request = PromptRequest {
        agent: Some(task.agent.clone()),
        model: task.model.clone(),
        no_reply: false,
        system: None,
        parts: vec![super::MessagePart {
            part_type: "text".to_string(),
            text: Some(input.prompt.clone()),
            tool: None,
            name: None,
            content: None,
        }],
    };

    let ok = client.session_prompt(&session_id, request).await;
    if !ok {
        return Err("Failed to prompt resume session".to_string());
    }

    Ok(task)
}

fn is_inside_tmux() -> bool {
    std::env::var("TMUX").is_ok()
}
