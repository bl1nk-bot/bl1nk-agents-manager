use crate::config::{AgentConfig, RoutingConfig, RoutingTier};
use crate::agents::register::{AgentRegistry, TaskStatus, TaskInfo};
use crate::rate_limit::RateLimitTracker;
use crate::agents::router::AgentRouter;
use crate::registry::RegistryService;
use anyhow::{Result, Context, bail};
use serde_json::Value;
// src/agents/executor.rs
//! ตัวดำเนินการเอเจนต์ (Agent Executor)
//! จัดการมอบหมายงานไปยังเอเจนต์ย่อยผ่านโปรโตคอล ACP
//! พร้อมการจัดการข้อผิดพลาด การป้องกัน deadlock และการล็อกที่ปลอดภัย

use crate::agents::{AgentRegistry, AgentRouter, register::{TaskInfo, TaskStatus}};
use crate::config::{AgentConfig, RoutingConfig, RoutingTier};
use crate::mcp::{DelegateTaskArgs, DelegateTaskOutput};
use crate::rate_limit::RateLimitTracker;
use anyhow::{Context, Result, bail};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::process::Command;
use crate::mcp::{DelegateTaskArgs, DelegateTaskOutput};

/// ตัวรันงานของเอเจนต์ (Agent Executor)
use tokio::process::{Command, ChildStdin, ChildStdout, ChildStderr};
use tokio::io::{AsyncWriteExt, AsyncBufReadExt, BufReader, AsyncReadExt};
use tokio::time::timeout;
use serde_json::Value;
use uuid::Uuid;
use sha2::{Sha256, Digest};

#[cfg(feature = "bundle-pmat")]
use pmat_core::run_context_analysis;

/// ระยะเวลา timeout สำหรับการอ่าน stdout และการรอ process (วินาที)
const STDOUT_READ_TIMEOUT: Duration = Duration::from_secs(30);
const PROCESS_WAIT_TIMEOUT: Duration = Duration::from_secs(10);

/// ตัวดำเนินการเอเจนต์หลัก
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
        }
    }

    /// คำนวณ hash ของข้อความเพื่อใช้ในการ log (ปลอดภัย)
    fn hash_prompt(prompt: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(prompt.as_bytes());
        let result = hasher.finalize();
        hex::encode(&result[..4]) // 8 ตัวอักษรแรกของ hash
    }

    /// มอบหมายงานไปยังเอเจนต์ย่อย
    pub async fn delegate_task(&self, args: DelegateTaskArgs) -> pmcp::Result<DelegateTaskOutput> {
        let task_id = Uuid::new_v4().to_string();
        let registry = self.agent_registry.read().await;

        let (agent_config, proposal) = if let Some(agent_id) = &args.agent_id {
            let config = registry.get_agent(agent_id)
                .ok_or_else(|| pmcp::Error::validation(format!("ไม่พบเอเจนต์: {}", agent_id)))?
                .clone();
            (config, None)
        } else {
            let agent_states = registry.get_agents_sorted();
            if args.interactive {
                let proposal = self.router.create_proposal(&args.task_type, &args.prompt, &agent_states)
                    .map_err(|e| pmcp::Error::internal(e.to_string()))?;
                let config = registry.get_agent(&proposal.agent_id)
                    .ok_or_else(|| pmcp::Error::internal("เอเจนต์ที่ถูกเลือกไม่อยู่ในรีจิสทรี"))?
                    .clone();
                (config, Some(proposal))
            } else {
                let agent_refs: Vec<&AgentConfig> = agents.iter().map(|&a| a).collect();
                self.router.select_agent(&args.task_type, &args.prompt, &agent_refs)?
                    .clone()
            }
        };

        // 2. จัดการ Task Registration
        let task_info = TaskInfo {
        let agent_id = agent_config.id.clone();
        let status = if args.interactive {
            TaskStatus::AwaitingApproval
        } else {
            TaskStatus::Pending
        };

        drop(registry);
        let mut registry = self.agent_registry.write().await;
        registry.register_task(TaskInfo {
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
        // ตรวจสอบ rate limit
        let mut rate_limiter = self.rate_limiter.write().await;
        if !rate_limiter.check_and_increment(&agent_id, &agent_config.rate_limit).await {
            // REVERT: ล้มเหลว อัปเดตสถานะ task เป็น Failed
            let mut registry = self.agent_registry.write().await;
            let _ = registry.update_task_status(&task_id, TaskStatus::Failed);
            drop(registry);
            return Err(pmcp::Error::internal(
                format!("เกินขีดจำกัดอัตราการเรียกสำหรับเอเจนต์: {}", agent_id)
            ));
        }
        drop(rate_limiter);

        if args.background {
            let executor = self.clone_for_background();
            let task_id_clone = task_id.clone();
            let agent_config_clone = agent_config.clone();
            let prompt_clone = args.prompt.clone();
            let context_clone = args.context.clone();

            tokio::spawn(async move {
                if let Err(e) = executor.execute_agent_task(
                    task_id_clone,
                    agent_config_clone,
                    prompt_clone,
                    context_clone,
                ).await {
                    tracing::error!("งานพื้นหลังล้มเหลว: {}", e);
                }
            });

            Ok(DelegateTaskOutput {
                task_id,
                agent_id,
                status: "pending".to_string(),
                result: None,
                proposal: None,
            })
        } else {
            let result = self.execute_agent_task(
                task_id.clone(),
                agent_config,
                args.prompt,
                args.context,
            ).await.map_err(|e| pmcp::Error::internal(e.to_string()))?;

            Ok(DelegateTaskOutput {
                task_id,
                agent_id,
                status: "completed".to_string(),
                result: Some(result),
                proposal: None,
            })
        }
    }

    /// อนุมัติงาน (แก้ไข race condition)
    pub async fn approve_task(&self, task_id: String, confirmed_agent_id: Option<String>) -> pmcp::Result<DelegateTaskOutput> {
        // ถือ lock registry ครั้งเดียวตลอดการตรวจสอบและอัปเดต
        let mut registry = self.agent_registry.write().await;
        let mut task = registry.get_task(&task_id)
            .ok_or_else(|| pmcp::Error::validation(format!("ไม่พบงาน: {}", task_id)))?
            .clone();

        if task.status != TaskStatus::AwaitingApproval {
            return Err(pmcp::Error::validation(format!("งาน {} ไม่อยู่ในสถานะรออนุมัติ", task_id)));
        }

        let agent_id = confirmed_agent_id.unwrap_or(task.agent_id.clone());
        let agent_config = registry.get_agent(&agent_id)
            .ok_or_else(|| pmcp::Error::validation(format!("ไม่พบเอเจนต์: {}", agent_id)))?
            .clone();

        // ตรวจสอบ rate limit ภายใต้ lock registry (แต่ต้อง ensure lock order)
        // เพื่อป้องกัน deadlock เรา drop registry ก่อนแล้วล็อก rate_limiter ใหม่
        // แต่วิธีที่ปลอดภัยกว่าคือใช้ try_lock หรือเปลี่ยนลำดับ แต่ที่นี่ใช้การตรวจสอบก่อนแล้วค่อยอัปเดต
        // เราใช้วิธี: ตรวจสอบ rate limit ก่อน แล้วค่อยอัปเดต task ภายใต้ lock เดียวกันอีกครั้ง
        drop(registry);
        let mut rate_limiter = self.rate_limiter.write().await;
        if !rate_limiter.check_and_increment(&agent_id, &agent_config.rate_limit).await {
            return Err(pmcp::Error::internal(
                format!("เกินขีดจำกัดอัตราการเรียกสำหรับเอเจนต์: {}", agent_id)
            ));
        }
        drop(rate_limiter);

        // อัปเดต task ด้วย agent ที่ยืนยัน
        let mut registry = self.agent_registry.write().await;
        // ดึง task อีกครั้งเพื่อความแน่ใจ (อาจมีคนเปลี่ยนระหว่างเรา drop lock)
        let mut task = registry.get_task(&task_id)
            .ok_or_else(|| pmcp::Error::validation(format!("ไม่พบงาน: {}", task_id)))?
            .clone();
        if task.status != TaskStatus::AwaitingApproval {
            return Err(pmcp::Error::validation(format!("งาน {} ไม่อยู่ในสถานะรออนุมัติอีกต่อไป", task_id)));
        }
        task.agent_id = agent_id.clone();
        task.status = TaskStatus::Pending;
        registry.register_task(task.clone());
        drop(registry);

        let result = self.execute_agent_task(
            task_id.clone(),
            agent_config,
            task.prompt,
            task.context,
        ).await.map_err(|e| pmcp::Error::internal(e.to_string()))?;

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
    async fn execute_agent_task(
        &self,
        task_id: String,
        agent: AgentConfig,
        prompt: String,
        context: Option<Value>,
    ) -> Result<String> {
        let mut registry = self.agent_registry.write().await;
        registry.update_task_status(&task_id, TaskStatus::Running)?;
        drop(registry);

        tracing::info!(
            "กำลังดำเนินงาน {} บนเอเจนต์ {} (prompt len={}, hash={})",
            task_id,
            agent.id,
            prompt.len(),
            Self::hash_prompt(&prompt)
        );

        let result = match agent.agent_type.as_str() {
            "cli" => self.execute_cli_agent(&agent, &prompt, context).await,
            "gemini-extension" => self.execute_gemini_extension(&agent, &prompt).await,
            "internal" if agent.command.as_deref() == Some("pmat-internal") => {
                self.execute_internal_pmat_agent(&prompt).await
            }
            "internal" => bail!("ไม่รองรับ internal agent: {:?}", agent.command),
            _ => bail!("ไม่รองรับประเภทเอเจนต์: {}", agent.agent_type),
        };

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
    #[cfg(feature = "bundle-pmat")]
    async fn execute_internal_pmat_agent(&self, prompt: &str) -> Result<String> {
        tracing::debug!(
            "เรียกใช้ bundled PMAT agent: prompt len={}, hash={}",
            prompt.len(),
            Self::hash_prompt(prompt)
        );
        run_context_analysis(prompt).await.map_err(|e| anyhow::anyhow!(e))
    }

    #[cfg(not(feature = "bundle-pmat"))]
    async fn execute_internal_pmat_agent(&self, _prompt: &str) -> Result<String> {
        bail!("ต้องเปิดใช้งานฟีเจอร์ 'bundle-pmat' หรือใช้ pmat CLI")
    }

    async fn execute_cli_agent(
        &self,
        agent: &AgentConfig,
        _prompt: &str,
        _context: Option<Value>,
    ) -> Result<String> {
        let command = &agent.command;
        let command = agent.command.as_ref().context("CLI agent ต้องการ command")?;
        tracing::debug!("spawn กระบวนการ: {} {:?}", command, agent.args);

        let mut child = Command::new(command)
            .args(agent.args.as_deref().unwrap_or(&[]))
            .spawn()
            .context("ไม่สามารถ spawn กระบวนการของเอเจนต์")?;

        let status = child.wait().await?;
        if status.success() {
            Ok("CLI task finished successfully".to_string())
        } else {
            bail!("CLI task failed with status: {}", status)
        }
    }

    pub fn clone_for_background(&self) -> Self {
        let stdin = child.stdin.take().context("ไม่พบ stdin")?;
        let stdout = child.stdout.take().context("ไม่พบ stdout")?;
        let stderr = child.stderr.take().context("ไม่พบ stderr")?;

        // Spawn task อ่าน stderr เพื่อไม่ให้ buffer เต็ม
        let stderr_handle = tokio::spawn(Self::drain_stderr(stderr));

        let acp_request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "context",
            "params": { "prompt": prompt, "context": context, "format": "llm-optimized" }
        });
        let request_line = serde_json::to_string(&acp_request)? + "\n";
        self.write_to_agent(stdin, &request_line).await?;

        // อ่านผลลัพธ์พร้อม timeout
        let response = timeout(STDOUT_READ_TIMEOUT, self.read_from_agent(stdout))
            .await
            .context("อ่าน stdout หมดเวลา")??;

        // รอ process จบพร้อม timeout
        let status = timeout(PROCESS_WAIT_TIMEOUT, child.wait())
            .await
            .context("รอ process จบหมดเวลา")??;

        // ยกเลิก stderr reader (ไม่ต้องรอ)
        stderr_handle.abort();

        if !status.success() {
            bail!("กระบวนการจบด้วยข้อผิดพลาด: {}", status);
        }
        Ok(response)
    }

    /// ระบาย stderr (เพื่อป้องกัน buffer เต็ม)
    async fn drain_stderr(stderr: ChildStderr) {
        let mut reader = BufReader::new(stderr);
        let mut line = String::new();
        while let Ok(n) = reader.read_line(&mut line).await {
            if n == 0 { break; }
            tracing::debug!("[stderr] {}", line.trim());
            line.clear();
        }
    }

    async fn write_to_agent(&self, mut stdin: ChildStdin, data: &str) -> Result<()> {
        stdin.write_all(data.as_bytes()).await?;
        stdin.flush().await?;
        Ok(())
    }

    async fn read_from_agent(&self, stdout: ChildStdout) -> Result<String> {
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        reader.read_line(&mut line).await?;
        let response: Value = serde_json::from_str(&line)?;
        if let Some(result) = response.get("result") {
            Ok(if result.is_string() { result.as_str().unwrap().to_string() } else { result.to_string() })
        } else if let Some(error) = response.get("error") {
            bail!("เอเจนต์ส่งค่าผิดพลาด: {}", error);
        } else {
            bail!("การตอบกลับ JSON-RPC ไม่ถูกต้อง");
        }
    }

    async fn execute_gemini_extension(&self, agent: &AgentConfig, _prompt: &str) -> Result<String> {
        let name = agent.extension_name.as_deref().unwrap_or("");
        tracing::info!("เรียกใช้ Gemini extension: {}", name);
        bail!("ยังไม่รองรับ Gemini extension")
    }

    fn clone_for_background(&self) -> Self {
        Self {
            agent_registry: self.agent_registry.clone(),
            rate_limiter: self.rate_limiter.clone(),
            router: AgentRouter::new(RoutingConfig { rules: vec![], tier: RoutingTier::Default }),
            weight_registry: self.weight_registry.clone(),
        }
    }
}