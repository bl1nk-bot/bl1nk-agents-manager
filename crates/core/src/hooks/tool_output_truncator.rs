use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ToolOutputTruncatorHook {
    // ใช้ cache สำหรับ session ต่างๆ
    session_cache: Arc<RwLock<HashMap<String, String>>>,
    truncate_all_tool_outputs: bool,
}

impl ToolOutputTruncatorHook {
    pub fn new(truncate_all_tool_outputs: bool) -> Self {
        Self {
            session_cache: Arc::new(RwLock::new(HashMap::new())),
            truncate_all_tool_outputs,
        }
    }

    pub async fn on_tool_execute_after(&self, tool_name: &str, session_id: &str, output: &mut String) {
        // ตรวจสอบว่าเครื่องมืออยู่ในรายการที่ต้องตัดหรือไม่
        let truncatable_tools = [
            "grep", "Grep", "safe_grep",
            "glob", "Glob", "safe_glob", 
            "lsp_diagnostics", "ast_grep_search",
            "interactive_bash", "Interactive_bash",
            "skill_mcp", "webfetch", "WebFetch"
        ];

        if !self.truncate_all_tool_outputs && !truncatable_tools.contains(&tool_name) {
            return;
        }

        // กำหนดขนาดสูงสุดตามประเภทเครื่องมือ
        let max_tokens = match tool_name {
            "webfetch" | "WebFetch" => 10_000, // ~40k chars
            _ => 50_000, // ~200k chars
        };

        // ตัดข้อมูลผลลัพธ์ตามขนาดที่กำหนด
        if self.should_truncate(output, max_tokens) {
            let truncated_output = self.truncate_output(output, max_tokens).await;
            *output = truncated_output;
            
            // บันทึกใน cache ถ้าจำเป็น
            let mut cache = self.session_cache.write().await;
            cache.insert(session_id.to_string(), output.clone());
        }
    }

    fn should_truncate(&self, output: &str, max_tokens: usize) -> bool {
        // ประมาณการจำนวน token โดยใช้ตัวอักษร (โดยประมาณ 1 token ≈ 4 ตัวอักษร)
        let estimated_tokens = output.chars().count() / 4;
        estimated_tokens > max_tokens
    }

    async fn truncate_output(&self, output: &str, max_tokens: usize) -> String {
        // ประมาณการจำนวนตัวอักษรที่อนุญาต
        let max_chars = max_tokens * 4;
        
        if output.len() <= max_chars {
            return output.to_string();
        }

        // ตัดข้อความและเพิ่มหมายเหตุว่าถูกตัด
        let truncated = output.chars().take(max_chars).collect::<String>();
        format!("{}\n\n[OUTPUT TRUNCATED: Content was too large and has been reduced for performance]", truncated)
    }
}