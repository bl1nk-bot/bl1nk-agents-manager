use crate::config::{AgentConfig, RoutingConfig, RoutingTier};
use crate::agents::register::{AgentRegistry, TaskStatus, TaskInfo};
use crate::rate_limit::RateLimitTracker;
use crate::agents::router::AgentRouter;
use crate::registry::RegistryService;
use anyhow::{Result, Context, bail};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::process::Command;
use crate::mcp::{DelegateTaskArgs, DelegateTaskOutput};

/// ตัวรันงานของเอเจนต์ (Agent Executor)
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
        let task_id = uuid::Uuid::new_v4().to_string();
        
        // 1. ค้นหาเอเจนต์
        let agent = {
            let registry = self.agent_registry.read().await;
            let agents = registry.get_agents_by_priority();
            
            if let Some(id) = &args.agent_id {
                (*agents.iter().find(|a| a.id == *id)
                    .context("Requested agent not found")?).clone()
            } else {
                let agent_refs: Vec<&AgentConfig> = agents.iter().map(|&a| a).collect();
                self.router.select_agent(&args.task_type, &args.prompt, &agent_refs)?
                    .clone()
            }
        };

        // 2. จัดการ Task Registration
        let task_info = TaskInfo {
            task_id: task_id.clone(),
            agent_id: agent.id.clone(),
            task_type: args.task_type.clone(),
            status: TaskStatus::Pending,
            proposal: None,
            prompt: args.prompt.clone(),
            context: args.context.clone(),
        };

        {
            let mut registry = self.agent_registry.write().await;
            registry.register_task(task_info);
        }

        // 3. รันงาน (ถ้าไม่ใช่ interactive)
        let mut result = None;
        let mut status = "pending".to_string();

        if !args.interactive {
            result = Some(self.execute_task_internal(&task_id, &agent, &args.prompt, args.context).await?);
            status = "completed".to_string();
        }

        Ok(DelegateTaskOutput {
            task_id,
            agent_id: agent.id,
            status,
            result,
            proposal: None,
        })
    }

    pub async fn approve_task(&self, task_id: String, _confirmed_agent_id: Option<String>) -> Result<DelegateTaskOutput> {
        // ดึงข้อมูลงานเดิม
        let (agent, prompt, context) = {
            let registry = self.agent_registry.read().await;
            let task = registry.get_task(&task_id).context("Task not found")?;
            let agent = registry.get_agent(&task.agent_id).context("Agent not found")?.clone();
            (agent, task.prompt.clone(), task.context.clone())
        };

        let result = self.execute_task_internal(&task_id, &agent, &prompt, context).await?;

        Ok(DelegateTaskOutput {
            task_id,
            agent_id: agent.id,
            status: "completed".to_string(),
            result: Some(result),
            proposal: None,
        })
    }

    async fn execute_task_internal(
        &self, 
        task_id: &str, 
        agent: &AgentConfig, 
        prompt: &str, 
        context: Option<Value>
    ) -> Result<String> {
        // --- POLICY ENFORCEMENT LAYER ---
        // ตรวจสอบว่าเอเจนต์มีสิทธิ์ตามกฎระเบียบหรือไม่
        let tool_name = match agent.agent_type.as_str() {
            "cli" => "bash",
            "internal" => "system",
            _ => "unknown"
        };

        let decision = crate::registry::PolicyEvaluator::evaluate(
            agent, 
            tool_name, 
            &context.clone().unwrap_or(serde_json::json!({}))
        );

        match decision {
            crate::registry::PolicyDecision::Deny => {
                bail!("❌ Security Violation: Agent '{}' is DENIED from using '{}' by policy.", agent.id, tool_name);
            },
            crate::registry::PolicyDecision::AskUser => {
                tracing::warn!("⚠️ Policy: Agent '{}' requires user approval for '{}'.", agent.id, tool_name);
                // ในเฟสนี้เราจะรันต่อ แต่จะบันทึก Log ไว้ (ในอนาคตจะใช้ ask_user tool)
            },
            crate::registry::PolicyDecision::Allow => {}
        }

        // ตรวจสอบ Rate Limit
        {
            let mut limiter = self.rate_limiter.write().await;
            if !limiter.check_and_increment(&agent.id, &agent.rate_limit).await {
                bail!("Rate limit exceeded for agent: {}", agent.id);
            }
        }

        // อัปเดตสถานะ
        // 3. รันงาน
        let mut registry = self.agent_registry.write().await;
        registry.update_task_status(task_id, TaskStatus::Running)?;
        drop(registry);

        let mut attempts = 0;
        let mut last_error = None;
        let mut result = Err(anyhow::anyhow!("Initial state"));

        while attempts < 3 {
            attempts += 1;
            let current_result = match agent.agent_type.as_str() {
                "cli" => self.execute_cli_agent(agent, prompt, context.clone()).await,
                "internal" => self.execute_internal_pmat_agent(prompt).await,
                _ => bail!("Unsupported agent type: {}", agent.agent_type),
            };

            if current_result.is_ok() {
                result = current_result;
                break;
            } else {
                last_error = Some(current_result.err().unwrap());
                tracing::warn!("⚠️ Attempt {} failed for agent {}", attempts, agent.id);
            }
        }

        // --- BEHAVIORAL ANALYSIS ---
        if attempts >= 3 && result.is_err() {
            let mut weights = self.weight_registry.write().await;
            // ถ้าพยายามครบ 3 ครั้งแล้วยังล้มเหลว และไม่มีการถามผู้ใช้ (ask_user)
            // ถือว่าเป็นการปกปิดข้อผิดพลาด (Hidden Error)
            weights.record_violation(&agent.id, crate::registry::ViolationType::HiddenError);
            tracing::error!("❌ Behavioral Violation: Agent '{}' hidden errors detected (3 fails).", agent.id);
        }

        // 4. อัปเดตสถานะสุดท้ายและบันทึกสถิติ
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
                    tracing::error!("Task failed: {}", e);
                },
            }
            // บันทึกสถิติลงไฟล์ทันที
            let _ = weights.save().await;
        }

        result
        }

    async fn execute_internal_pmat_agent(&self, _prompt: &str) -> Result<String> {
        Ok("PMAT analysis complete (simulated)".to_string())
    }

    async fn execute_cli_agent(
        &self,
        agent: &AgentConfig,
        _prompt: &str,
        _context: Option<Value>,
    ) -> Result<String> {
        let command = &agent.command;
        let mut child = Command::new(command)
            .args(agent.args.as_deref().unwrap_or(&[]))
            .spawn()
            .context("Failed to spawn agent process")?;

        let status = child.wait().await?;
        if status.success() {
            Ok("CLI task finished successfully".to_string())
        } else {
            bail!("CLI task failed with status: {}", status)
        }
    }

    pub fn clone_for_background(&self) -> Self {
        Self {
            agent_registry: self.agent_registry.clone(),
            rate_limiter: self.rate_limiter.clone(),
            router: AgentRouter::new(RoutingConfig { rules: vec![], tier: RoutingTier::Default }),
            weight_registry: self.weight_registry.clone(),
        }
    }
}
