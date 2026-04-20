// src/tools/exit_plan_mode.rs
use anyhow::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

/// พารามิเตอร์สำหรับเครื่องมือ ExitPlanMode (v1.7.5.1)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExitPlanArgs {
    #[schemars(description = "สรุปแผนงานขั้นสุดท้ายที่จะนำเสนอให้ผู้ใช้")]
    pub plan_summary: String,
    #[schemars(description = "โหมดที่ต้องการเข้าสู่หลังจากการวางแผนเสร็จสิ้น")]
    pub next_mode: Option<String>,
}

/// ผลลัพธ์จากการรันเครื่องมือ
#[derive(Debug, Serialize, JsonSchema)]
pub struct ExitPlanOutput {
    pub success: bool,
    pub message: String,
}

/// ตัวจัดการเครื่องมือ ExitPlanMode
/// แปลงมาจาก TypeScript Implementation เพื่อความปลอดภัยและเสถียรภาพใน Rust
pub struct ExitPlanModeHandler;

impl ExitPlanModeHandler {
    pub fn new() -> Self {
        Self
    }

    /// รันการเปลี่ยนโหมดอย่างปลอดภัย (เทียบเท่า setApprovalModeSafely)
    pub async fn execute(&self, args: ExitPlanArgs) -> Result<ExitPlanOutput> {
        debug!("🛠️ [EXIT_PLAN_MODE] Closing plan mode. Summary length: {}", args.plan_summary.len());

        // การจัดการความปลอดภัยในการเปลี่ยนสถานะ (Safety Layer)
        match self.update_runtime_state(&args.next_mode).await {
            Ok(_) => {
                Ok(ExitPlanOutput {
                    success: true,
                    message: format!("✅ แผนงานได้รับการบันทึกแล้ว: {}", args.plan_summary),
                })
            }
            Err(e) => {
                error!("❌ [EXIT_PLAN_MODE] Safety check failed: {}", e);
                Ok(ExitPlanOutput {
                    success: false,
                    message: format!("⚠️ ไม่สามารถปิดโหมดวางแผนได้: {}", e),
                })
            }
        }
    }

    async fn update_runtime_state(&self, _mode: &Option<String>) -> Result<()> {
        // TODO: เชื่อมต่อกับระบบจัดการสถานะของเอเจนต์หลัก
        Ok(())
    }
}
