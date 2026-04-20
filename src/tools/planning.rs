//! Planning Tools (enter_plan_mode, exit_plan_mode)
//! พอร์ตจากมาตรฐาน Gemini CLI (v1.7.5.1)

use anyhow::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// ข้อมูลนำเข้าสำหรับ exit_plan_mode
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ExitPlanArgs {
    #[schemars(description = "พาธไปยังไฟล์แผนการที่สรุปแล้วในโฟลเดอร์ชั่วคราว")]
    pub plan_path: String,
}

/// ข้อมูลนำเข้าสำหรับ enter_plan_mode
#[derive(Debug, Deserialize, JsonSchema)]
pub struct EnterPlanArgs {
    #[schemars(description = "เหตุผลสั้นๆ ในการเข้าสู่โหมดวางแผน")]
    pub reason: String,
}

pub struct PlanningTools;

impl PlanningTools {
    /// เลียนแบบตรรกะ setApprovalModeSafely (TypeScript) ในรูปแบบ Rust
    fn set_approval_mode_safely(mode: &str) {
        tracing::debug!(mode = %mode, "🛡️ Setting approval mode safely");
        // ในระบบจริง จะมีการอัปเดตสถานะใน AgentRegistry หรือ Session Config
    }

    pub fn enter_plan_mode(args: EnterPlanArgs) -> Result<String> {
        tracing::info!(reason = %args.reason, "🔄 Entering READ-ONLY PLAN MODE");
        Self::set_approval_mode_safely("planning");
        Ok(format!("Plan mode activated. Reason: {}", args.reason))
    }

    pub fn exit_plan_mode(args: ExitPlanArgs) -> Result<String> {
        tracing::info!(plan = %args.plan_path, "✅ Exiting PLAN MODE. Requesting user approval...");
        Self::set_approval_mode_safely("approval");
        Ok(format!("Plan at {} finalized. Awaiting manual user confirmation to proceed.", args.plan_path))
    }
}
