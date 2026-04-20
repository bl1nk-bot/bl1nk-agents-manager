//! 🛠️ BL1NK Universal Toolset (v1.7.5.1)
//!
//! โมดูลจัดการเครื่องมือมาตรฐานสำหรับเอเจนต์ รองรับทั้ง Gemini CLI และ KiloCode standards

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

pub mod ask_user_question;
pub mod bash;
pub mod exit_plan_mode;
pub mod file_ops;

pub use ask_user_question::AskUserQuestionInput;
pub use exit_plan_mode::ExitPlanModeHandler;
pub use file_ops::FileOpsTools;

/// Trait มาตรฐานสำหรับเครื่องมือของเอเจนต์
#[async_trait]
pub trait AgentTool: Send + Sync {
    /// ชื่อเครื่องมือ (เช่น "read_file" หรือ "write_file")
    fn name(&self) -> &str;
    
    /// คำอธิบายความสามารถ
    fn description(&self) -> &str;
    
    /// รันเครื่องมือด้วย JSON input และคืนค่าเป็น JSON output
    async fn execute(&self, input: serde_json::Value) -> Result<serde_json::Value>;
}

/// ข้อมูลสรุปของเครื่องมือสำหรับส่งให้ Model
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}
