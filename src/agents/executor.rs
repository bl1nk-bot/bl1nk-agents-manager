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
        // 1. สร้าง Proposal ผ่าน Router
        let proposal = {
            let registry = self.agent_registry.read().await;
            self.router.route_task(&registry, &args.task_type, &args.prompt).await?
        };

        let task_id = proposal.task_id.clone();
        let agent_id = proposal.agent_id.clone();

        // 2. ลงทะเบียน Task
        let task_info = TaskInfo {
            task_id: task_id.clone(),
            agent_id: agent_id.clone(),
            task_type: args.task_type.clone(),
            status: if args.interactive { TaskStatus::AwaitingApproval } else { TaskStatus::Pending },
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
            // โหมด Interactive: คืนข้อเสนอให้ผู้ใช้ตัดสินใจ
            Ok(DelegateTaskOutput {
                task_id,
                agent_id,
                status: "awaiting_approval".to_string(),
                result: None,
                proposal: Some(proposal),
            })
        } else {
            // โหมดปกติ: รันทันที
            let agent = {
                let registry = self.agent_registry.read().await;
                registry.get_agent(&agent_id).context("Agent not found")?.clone()
            };

            let result = self.execute_task_internal(&task_id, &agent, &args.prompt, args.context).await?;

            Ok(DelegateTaskOutput {
                task_id,
                agent_id,
                status: "completed".to_string(),
                result: Some(result),
                proposal: None,
            })
        }
    }

    pub async fn approve_task(&self, task_id: String, confirmed_agent_id: Option<String>) -> Result<DelegateTaskOutput> {
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
        // อัปเดตสถานะเป็น Running
        {
            let mut registry = self.agent_registry.write().await;
            registry.update_task_status(task_id, TaskStatus::Running)?;
        }

        // ตรวจสอบความปลอดภัยเบื้องต้น
        if agent.permission < 100 {
            bail!("Agent permissions too low");
        }

        // จำลองการรันงาน
        let result = match agent.agent_type.as_str() {
            "cli" => self.execute_cli_agent(agent, prompt, context).await,
            _ => Ok(format!("Task executed by {} (simulated)", agent.name)),
        };

        // อัปเดตผลลัพธ์
        let mut registry = self.agent_registry.write().await;
        match &result {
            Ok(_) => registry.update_task_status(task_id, TaskStatus::Completed)?,
            Err(_) => registry.update_task_status(task_id, TaskStatus::Failed)?,
        }

        result
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
            .context("Failed to spawn CLI agent")?;

        let status = child.wait().await?;
        if status.success() {
            Ok("CLI task finished successfully".to_string())
        } else {
            bail!("CLI task failed")
        }
    }
}
