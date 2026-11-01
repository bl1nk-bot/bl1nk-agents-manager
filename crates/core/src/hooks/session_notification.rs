use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct SessionNotificationHook {
    // เก็บสถานะการแจ้งเตือนของแต่ละ session
    notification_states: Arc<RwLock<HashMap<String, NotificationState>>>,
}

#[derive(Debug, Clone)]
pub struct NotificationState {
    pub last_notification_time: Option<std::time::Instant>,
    pub notification_count: u32,
    pub is_active: bool,
}

impl SessionNotificationHook {
    pub fn new() -> Self {
        Self {
            notification_states: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn on_session_start(&self, session_id: &str) {
        let mut states = self.notification_states.write().await;
        states.insert(
            session_id.to_string(),
            NotificationState {
                last_notification_time: Some(std::time::Instant::now()),
                notification_count: 0,
                is_active: true,
            },
        );
    }

    pub async fn on_session_end(&self, session_id: &str) {
        let mut states = self.notification_states.write().await;
        if let Some(mut state) = states.get_mut(session_id) {
            state.is_active = false;
        }
    }

    pub async fn send_notification(&self, session_id: &str, title: &str, message: &str) -> Result<(), Box<dyn std::error::Error>> {
        // ตรวจสอบว่า session ยังใช้งานอยู่หรือไม่
        let is_active = {
            let states = self.notification_states.read().await;
            states.get(session_id).map(|state| state.is_active).unwrap_or(false)
        };

        if !is_active {
            return Ok(());
        }

        // อัปเดตสถานะการแจ้งเตือน
        {
            let mut states = self.notification_states.write().await;
            if let Some(mut state) = states.get_mut(session_id) {
                state.notification_count += 1;
                state.last_notification_time = Some(std::time::Instant::now());
            }
        }

        // แสดงการแจ้งเตือน (ในตัวอย่างนี้ใช้ println แต่ในระบบจริงอาจใช้ระบบแจ้งเตือนจริง)
        println!("[NOTIFICATION - {}] {}: {}", session_id, title, message);

        Ok(())
    }

    pub async fn get_notification_stats(&self, session_id: &str) -> Option<NotificationState> {
        let states = self.notification_states.read().await;
        states.get(session_id).cloned()
    }

    pub async fn clear_notifications(&self, session_id: &str) {
        let mut states = self.notification_states.write().await;
        states.remove(session_id);
    }

    pub async fn on_tool_execute_after(&self, session_id: &str, tool_name: &str, output: &mut String) {
        // ตรวจสอบว่ามีข้อผิดพลาดหรือสถานะที่ต้องแจ้งเตือนหรือไม่
        if output.contains("ERROR") || output.contains("WARNING") {
            let _ = self.send_notification(
                session_id,
                "Tool Execution Alert",
                &format!("Tool '{}' completed with alerts: {}", tool_name, output)
            ).await;
        }
    }
}