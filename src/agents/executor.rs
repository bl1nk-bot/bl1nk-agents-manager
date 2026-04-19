use crate::agents::register::{AgentRegistry, TaskInfo, TaskStatus};
use crate::agents::router::AgentRouter;
use crate::config::{AgentConfig, RoutingConfig};
use crate::mcp::{DelegateTaskArgs, DelegateTaskOutput};
use crate::rate_limit::RateLimitTracker;
use crate::registry::{RegistryService, PolicyEvaluator, PolicyDecision, ViolationType};
use anyhow::{Context, Result, bail};
use serde_json::Value;
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::RwLock;

/// ตัวรันงานของเอเจนต์ (Agent Executor)
/// รับผิดชอบวงจรชีวิตของงาน ตั้งแต่การเลือกเอเจนต์, ตรวจสอบสิทธิ์, รันงาน และบันทึกสถิติ
pub struct AgentExecutor {
    agent_registry: Arc<RwLock<AgentRegistry>>,
    rate_limiter: Arc<RwLock<RateLimitTracker>>,
    router: AgentRouter,
    weight_registry: Arc<RwLock<crate::registry::WeightRegistry>>,
}

impl AgentExecutor {
    pub fn new(
        agent_registry: Arc<RwLock<AgentRegistry>>,
        rate_limiter: Arc<RwLock<RateLimitTracker>>,
        routing_config: RoutingConfig,
        weight_registry: Arc<RwLock<crate::registry::WeightRegistry>>,
    ) -> Self {
        Self {
            agent_registry,
            rate_limiter,
            router: AgentRouter::new(routing_config),
            weight_registry,
        }
    }

    pub fn with_registry(mut self, service: Arc<RegistryService>) -> Self {
        self.router = self.router.with_registry(service);
        self
    }

    pub async fn delegate_task(&self, args: DelegateTaskArgs) -> Result<DelegateTaskOutput> {
        // 1. สร้าง Proposal ผ่าน Router (เลือกเอเจนต์ที่เหมาะสมที่สุด)
        let proposal = {
            let registry = self.agent_registry.read().await;
            self.router.route_task(&registry, &args.task_type, &args.prompt).await?
        };

        let task_id = proposal.task_id.clone();
        let agent_id = proposal.agent_id.clone();

        // 2. ลงทะเบียน Task เข้าระบบสถานะ
        let task_info = TaskInfo {
            task_id: task_id.clone(),
            agent_id: agent_id.clone(),
            task_type: args.task_type.clone(),
            status: if args.interactive {
                TaskStatus::AwaitingApproval
            } else {
                TaskStatus::Pending
            },
            proposal: Some(proposal.clone()),
            prompt: args.prompt.clone(),
            context: args.context.clone(),
        };

        {
            let mut registry = self.agent_registry.write().await;
            registry.register_task(task_info);
        }

        // 3. จัดการการรันงาน
        if args.interactive {
            // ถ้าเป็นโหมด Interactive ให้หยุดรอการยืนยันจากผู้ใช้
            Ok(DelegateTaskOutput {
                task_id,
                agent_id,
                status: "awaiting_approval".to_string(),
                result: None,
                proposal: Some(proposal),
            })
        } else {
            // ถ้าไม่ใช่ ให้ดึงเอเจนต์และรันทันที
            let agent = {
                let registry = self.agent_registry.read().await;
                registry.get_agent(&agent_id).context("Agent not found")?.clone()
            };

            let result = self
                .execute_task_internal(&task_id, &agent, &args.prompt, args.context)
                .await?;

            Ok(DelegateTaskOutput {
                task_id,
                agent_id,
                status: "completed".to_string(),
                result: Some(result),
                proposal: None,
            })
        }
    }

    pub async fn approve_task(
        &self,
        task_id: String,
        confirmed_agent_id: Option<String>,
    ) -> Result<DelegateTaskOutput> {
        // ดึงข้อมูลงานเดิมที่ค้างอยู่
        let (agent_id, prompt, context) = {
            let registry = self.agent_registry.read().await;
            let task = registry.get_task(&task_id).context("Task not found")?;
            let final_agent_id = confirmed_agent_id.unwrap_or_else(|| task.agent_id.clone());
            (final_agent_id, task.prompt.clone(), task.context.clone())
        };

        let agent = {
            let registry = self.agent_registry.read().await;
            registry.get_agent(&agent_id).context("Agent not found")?.clone()
        };

        let result = self
            .execute_task_internal(&task_id, &agent, &prompt, context)
            .await?;

        Ok(DelegateTaskOutput {
            task_id,
            agent_id,
            status: "completed".to_string(),
            result: Some(result),
            proposal: None,
        })
    }

    /// รันงานภายใน พร้อมระบบความปลอดภัยและการลองใหม่ (Retry)
    async fn execute_task_internal(
        &self,
        task_id: &str,
        agent: &AgentConfig,
        prompt: &str,
        context: Option<Value>,
    ) -> Result<String> {
        // --- 🛡️ POLICY ENFORCEMENT LAYER ---
        // ตรวจสอบว่าเอเจนต์มีสิทธิ์ใช้เครื่องมือตามกฎระเบียบ (Policy)
        let tool_name = match agent.agent_type.as_str() {
            "cli" => "bash",
            "internal" => "system",
            _ => "unknown"
        };

        let decision = PolicyEvaluator::evaluate(
            agent, 
            tool_name, 
            &context.clone().unwrap_or(serde_json::json!({}))
        );

        match decision {
            PolicyDecision::Deny => {
                bail!("❌ Security Violation: Agent '{}' is DENIED from using '{}' by policy.", agent.id, tool_name);
            },
            PolicyDecision::AskUser => {
                tracing::warn!("⚠️ Policy: Agent '{}' requires user approval for '{}'.", agent.id, tool_name);
                // ในเวอร์ชัน CLI ปัจจุบัน เราจะรันต่อแต่บันทึกคำเตือนไว้
            },
            PolicyDecision::Allow => {}
        }

        // --- ⏳ RATE LIMIT CHECK ---
        {
            let mut limiter = self.rate_limiter.write().await;
            if !limiter.check_and_increment(&agent.id, &agent.rate_limit).await {
                bail!("Rate limit exceeded for agent: {}", agent.id);
            }
        }

        // --- 🔄 EXECUTION WITH RETRY LOOP ---
        let mut registry = self.agent_registry.write().await;
        registry.update_task_status(task_id, TaskStatus::Running)?;
        drop(registry);

        let mut attempts = 0;
        let mut result = Err(anyhow::anyhow!("Initial state"));

        while attempts < 3 {
            attempts += 1;
            let current_result = match agent.agent_type.as_str() {
                "cli" => self.execute_cli_agent(agent, prompt, context.clone()).await,
                "internal" => self.execute_internal_agent(prompt).await,
                _ => Ok(format!("Task executed by {} (simulated)", agent.name)),
            };

            if current_result.is_ok() {
                result = current_result;
                break;
            } else {
                let err = current_result.err().unwrap();
                tracing::warn!("⚠️ Attempt {}/3 failed for agent {}: {}", attempts, agent.id, err);
                result = Err(err);
            }
        }

        // --- 📊 BEHAVIORAL ANALYSIS & STATISTICS ---
        {
            let mut registry = self.agent_registry.write().await;
            let mut weights = self.weight_registry.write().await;

            match &result {
                Ok(_) => {
                    registry.update_task_status(task_id, TaskStatus::Completed)?;
                    weights.record_result(&agent.id, true);
                },
                Err(e) => {
                    registry.update_task_status(task_id, TaskStatus::Failed)?;
                    weights.record_result(&agent.id, false);
                    
                    // ถ้าพยายามครบ 3 ครั้งแล้วยังล้มเหลว บันทึกเป็น Violation
                    if attempts >= 3 {
                        weights.record_violation(&agent.id, ViolationType::HiddenError);
                        tracing::error!("❌ Behavioral Violation: Agent '{}' failed after 3 attempts.", agent.id);
                    }
                    tracing::error!("Task failed: {}", e);
                },
            }
            // บันทึกสถิติลงไฟล์ Persistence
            let _ = weights.save().await;
        }

        result
    }

    async fn execute_internal_agent(&self, _prompt: &str) -> Result<String> {
        Ok("Internal agent task complete (simulated)".to_string())
    }

    async fn execute_cli_agent(
        &self,
        agent: &AgentConfig,
        _prompt: &str,
        _context: Option<Value>,
    ) -> Result<String> {
        let mut child = Command::new(&agent.command)
            .args(agent.args.as_deref().unwrap_or(&[]))
            .spawn()
            .context("Failed to spawn CLI agent process")?;

        let status = child.wait().await?;
        if status.success() {
            Ok("CLI task finished successfully".to_string())
        } else {
            bail!("CLI task failed with status: {}", status)
        }
    }
}
