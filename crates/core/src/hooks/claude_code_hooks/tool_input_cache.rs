use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
struct CacheEntry {
    tool_input: serde_json::Value,
    timestamp: u64,
}

#[derive(Clone)]
pub struct ToolInputCache {
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    ttl_ms: u64,
}

impl ToolInputCache {
    pub fn new(ttl_ms: Option<u64>) -> Self {
        let ttl = ttl_ms.unwrap_or(60000); // 1 นาทีเป็น default
        let cache = Arc::new(RwLock::new(HashMap::new()));
        
        // เริ่ม task สำหรับทำความสะอาด cache อย่างต่อเนื่อง
        let cache_clone = cache.clone();
        let ttl_clone = ttl;
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_millis(ttl_clone)).await;
                cleanup_expired_entries(&cache_clone, ttl_clone).await;
            }
        });
        
        Self {
            cache,
            ttl_ms: ttl,
        }
    }

    pub async fn cache_tool_input(
        &self,
        session_id: &str,
        tool_name: &str,
        invocation_id: &str,
        tool_input: serde_json::Value,
    ) {
        let key = format!("{}:{}:{}", session_id, tool_name, invocation_id);
        let timestamp = get_current_timestamp();
        
        let mut cache = self.cache.write().await;
        cache.insert(
            key,
            CacheEntry {
                tool_input,
                timestamp,
            },
        );
    }

    pub async fn get_tool_input(
        &self,
        session_id: &str,
        tool_name: &str,
        invocation_id: &str,
    ) -> Option<serde_json::Value> {
        let key = format!("{}:{}:{}", session_id, tool_name, invocation_id);
        
        let mut cache = self.cache.write().await; // ใช้ write lock เพื่อสามารถลบได้
        if let Some(entry) = cache.remove(&key) {
            let current_time = get_current_timestamp();
            if current_time - entry.timestamp <= self.ttl_ms {
                Some(entry.tool_input.clone())
            } else {
                // ถ้า expired แล้ว ลบออกจาก cache และคืนค่า null
                None
            }
        } else {
            None
        }
    }
}

async fn cleanup_expired_entries(cache: &Arc<RwLock<HashMap<String, CacheEntry>>>, ttl_ms: u64) {
    let current_time = get_current_timestamp();
    let mut cache_write = cache.write().await;
    
    cache_write.retain(|_, entry| {
        current_time - entry.timestamp <= ttl_ms
    });
}

fn get_current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis() as u64
}

impl Default for ToolInputCache {
    fn default() -> Self {
        Self::new(Some(60000)) // 1 นาที
    }
}

// Standalone function for compatibility
pub async fn get_tool_input(
    _session_id: &str,
    _tool_name: &str,
    _call_id: &str,
) -> Option<serde_json::Value> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tool_input_cache() {
        let cache = ToolInputCache::new(Some(1000)); // TTL 1 วินาทีสำหรับการทดสอบ
        
        let session_id = "test_session";
        let tool_name = "test_tool";
        let invocation_id = "test_invocation";
        let tool_input = serde_json::json!({"param": "value"});
        
        // บันทึกข้อมูล
        cache.cache_tool_input(session_id, tool_name, invocation_id, tool_input.clone()).await;
        
        // ดึงข้อมูล
        let retrieved = cache.get_tool_input(session_id, tool_name, invocation_id).await;
        assert_eq!(retrieved, Some(tool_input));
        
        // ดึงอีกครั้ง ควรได้ None เพราะถูกลบแล้ว
        let retrieved_again = cache.get_tool_input(session_id, tool_name, invocation_id).await;
        assert_eq!(retrieved_again, None);
    }

    #[tokio::test]
    async fn test_expired_entry() {
        let cache = ToolInputCache::new(Some(10)); // TTL 10ms สำหรับการทดสอบ
        
        let session_id = "test_session_2";
        let tool_name = "test_tool_2";
        let invocation_id = "test_invocation_2";
        let tool_input = serde_json::json!({"param": "value2"});
        
        // บันทึกข้อมูล
        cache.cache_tool_input(session_id, tool_name, invocation_id, tool_input).await;
        
        // รอให้หมดอายุ
        tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
        
        // ดึงข้อมูล ควรได้ None เพราะหมดอายุ
        let retrieved = cache.get_tool_input(session_id, tool_name, invocation_id).await;
        assert_eq!(retrieved, None);
    }
}