use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundTask {
    pub id: String,
    pub status: String,
}

pub struct BackgroundNotificationHook;

impl BackgroundNotificationHook {
    pub fn new() -> Self { Self }

    pub fn format_notification(&self, task: &BackgroundTask) -> String {
        format!("[Background Task] {} is now {}", task.id, task.status)
    }
}
