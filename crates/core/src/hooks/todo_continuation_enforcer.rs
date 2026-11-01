use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct TodoContinuationEnforcerHook {
    // สถานะการกู้คืนของแต่ละ session
    is_recovering: Arc<RwLock<HashMap<String, bool>>>,
    // ตัวนับเวลาถอยหลัง
    countdown_timers: Arc<RwLock<HashMap<String, tokio::time::Instant>>>,
    // ตัวแปรเก็บจำนวน todo ที่ยังไม่เสร็จ
    incomplete_counts: Arc<RwLock<HashMap<String, usize>>>,
    // ตัวแปรเก็บ agent ที่ข้าม
    skip_agents: Vec<String>,
}

impl TodoContinuationEnforcerHook {
    pub fn new(skip_agents: Option<Vec<String>>) -> Self {
        let default_skip_agents = vec![
            "prometheus".to_string(),
            "compaction".to_string(),
        ];
        
        Self {
            is_recovering: Arc::new(RwLock::new(HashMap::new())),
            countdown_timers: Arc::new(RwLock::new(HashMap::new())),
            incomplete_counts: Arc::new(RwLock::new(HashMap::new())),
            skip_agents: skip_agents.unwrap_or(default_skip_agents),
        }
    }

    pub async fn mark_recovering(&self, session_id: &str) {
        let mut recovering = self.is_recovering.write().await;
        recovering.insert(session_id.to_string(), true);
        
        // ยกเลิกตัวนับเวลาถอยหลัง
        let mut timers = self.countdown_timers.write().await;
        timers.remove(session_id);
    }

    pub async fn mark_recovery_complete(&self, session_id: &str) {
        let mut recovering = self.is_recovering.write().await;
        recovering.insert(session_id.to_string(), false);
    }

    pub async fn on_session_idle(&self, session_id: &str, agent_name: Option<&str>) -> Option<String> {
        // ตรวจสอบว่า session อยู่ในสถานะกู้คืนหรือไม่
        {
            let recovering = self.is_recovering.read().await;
            if recovering.get(session_id) == Some(&true) {
                return None;
            }
        }

        // ตรวจสอบว่า agent อยู่ในรายการที่ข้ามหรือไม่
        if let Some(agent) = agent_name {
            if self.skip_agents.contains(&agent.to_lowercase()) {
                return None;
            }
        }

        // ตรวจสอบจำนวน todo ที่ยังไม่เสร็จ
        let incomplete_count = {
            let counts = self.incomplete_counts.read().await;
            *counts.get(session_id).unwrap_or(&0)
        };

        if incomplete_count == 0 {
            return None;
        }

        // เริ่มตัวนับเวลาถอยหลัง
        self.start_countdown(session_id, incomplete_count).await;

        None
    }

    async fn start_countdown(&self, session_id: &str, incomplete_count: usize) {
        // บันทึกเวลาเริ่มต้น
        let mut timers = self.countdown_timers.write().await;
        timers.insert(session_id.to_string(), tokio::time::Instant::now());

        // สร้าง task สำหรับตัวนับเวลาถอยหลัง
        let is_recovering_clone = self.is_recovering.clone();
        let session_id_clone = session_id.to_string();
        
        tokio::spawn(async move {
            // รอ 2 วินาที
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            
            // ตรวจสอบว่า session ยังไม่อยู่ในสถานะกู้คืน
            let recovering = is_recovering_clone.read().await;
            if recovering.get(&session_id_clone) != Some(&true) {
                // สร้าง prompt สำหรับการดำเนินการต่อ
                println!("[TODO CONTINUATION] Resuming work on {} incomplete tasks", incomplete_count);
            }
        });
    }

    pub async fn update_incomplete_count(&self, session_id: &str, count: usize) {
        let mut counts = self.incomplete_counts.write().await;
        counts.insert(session_id.to_string(), count);
    }

    pub async fn on_event(&self, event_type: &str, session_id: Option<&str>) {
        if let Some(session_id) = session_id {
            match event_type {
                "session.deleted" => {
                    // ล้างข้อมูลที่เกี่ยวข้องกับ session
                    let mut recovering = self.is_recovering.write().await;
                    recovering.remove(session_id);
                    
                    let mut timers = self.countdown_timers.write().await;
                    timers.remove(session_id);
                    
                    let mut counts = self.incomplete_counts.write().await;
                    counts.remove(session_id);
                },
                "message.updated" | "tool.execute.before" | "tool.execute.after" => {
                    // ยกเลิกตัวนับเวลาถอยหลังเมื่อมีการอัปเดต
                    let mut timers = self.countdown_timers.write().await;
                    timers.remove(session_id);
                },
                _ => {}
            }
        }
    }
}