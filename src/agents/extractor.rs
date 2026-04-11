// src/agents/executor.rs
//! ตัวดำเนินการเอเจนต์ (Agent Executor)
//!
//! จัดการการมอบหมายงานไปยังเอเจนต์ย่อยผ่านโปรโตคอล ACP
//! รองรับการเลือกเอเจนต์อัตโนมัติ การตรวจสอบ rate limit และการทำงานแบบ background
//! รวมถึงการอนุมัติงานที่รอการยืนยัน (interactive mode)

// นำเข้าจากโมดูล agents
use crate::agents::{AgentRegistry, AgentRouter, register::{TaskInfo, TaskStatus}};
// นำเข้าจากโมดูล config
use crate::config::{AgentConfig, RoutingConfig, RoutingTier};
use crate::mcp::{DelegateTaskArgs, DelegateTaskOutput};
use crate::rate_limit::RateLimitTracker;
use anyhow::{Context, Result, bail};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::process::{Command, ChildStdin, ChildStdout};
use tokio::io::{AsyncWriteExt, AsyncBufReadExt, BufReader};
use serde_json::Value;
use uuid::Uuid;

// นำเข้าแบบมีเงื่อนไขสำหรับ bundled pmat
#[cfg(feature = "bundle-pmat")]
use pmat_core::run_context_analysis; // สมมติว่า pmat-core มีฟังก์ชันนี้

/// ตัวดำเนินการเอเจนต์หลัก
///
/// รับผิดชอบ:
/// - การลงทะเบียนและติดตามสถานะของ task
/// - การเลือกเอเจนต์ที่เหมาะสม (ผ่าน AgentRouter)
/// - การตรวจสอบ rate limit
/// - การรันเอเจนต์ในรูปแบบต่างๆ (CLI, Gemini extension, internal)
pub struct AgentExecutor {
    /// รีจิสทรีของเอเจนต์ (แชร์ระหว่างหลายส่วนของระบบ)
    agent_registry: Arc<RwLock<AgentRegistry>>,
    /// ตัวติดตาม rate limit (แชร์)
    rate_limiter: Arc<RwLock<RateLimitTracker>>,
    /// เราเตอร์สำหรับเลือกเอเจนต์ตามกฎ
    router: AgentRouter,
}

impl AgentExecutor {
    /// สร้าง AgentExecutor ใหม่
    ///
    /// # Arguments
    /// * `agent_registry` - รีจิสทรีของเอเจนต์ที่ลงทะเบียนไว้
    /// * `rate_limiter` - ตัวติดตาม rate limit
    /// * `routing_config` - คอนฟิกสำหรับการเลือกเส้นทางเอเจนต์
    pub fn new(
        agent_registry: Arc<RwLock<AgentRegistry>>,
        rate_limiter: Arc<RwLock<RateLimitTracker>>,
        routing_config: RoutingConfig,
    ) -> Self {
        let router = AgentRouter::new(routing_config);
        Self {
            agent_registry,
            rate_limiter,
            router,
        }
    }

    /// มอบหมายงานไปยังเอเจนต์ย่อยผ่าน ACP
    ///
    /// # Arguments
    /// * `args` - อาร์กิวเมนต์การมอบหมายงาน (ประกอบด้วย task_type, prompt, agent_id ตัวเลือก, interactive flag, background flag)
    ///
    /// # Returns
    /// * `pmcp::Result<DelegateTaskOutput>` - ผลลัพธ์การมอบหมายงาน
    pub async fn delegate_task(&self, args: DelegateTaskArgs) -> pmcp::Result<DelegateTaskOutput> {
        // สร้าง task ID ใหม่
        let task_id = Uuid::new_v4().to_string();

        // อ่านรีจิสทรีเพื่อเลือกเอเจนต์หรือสร้างข้อเสนอ
        let registry = self.agent_registry.read().await;

        // เลือกเอเจนต์ตามที่ระบุหรืออัตโนมัติ
        let (agent_config, proposal) = if let Some(agent_id) = &args.agent_id {
            // ระบุเอเจนต์โดยตรง
            let config = registry.get_agent(agent_id)
                .ok_or_else(|| pmcp::Error::validation(format!("ไม่พบเอเจนต์: {}", agent_id)))?
                .clone();
            (config, None)
        } else {
            // เลือกอัตโนมัติตาม task_type
            let agent_states = registry.get_agents_sorted();

            if args.interactive {
                // โหมดโต้ตอบ: สร้างข้อเสนอให้ผู้ใช้ยืนยัน
                let proposal = self.router.create_proposal(&args.task_type, &args.prompt, &agent_states)
                    .map_err(|e| pmcp::Error::internal(e.to_string()))?;

                let config = registry.get_agent(&proposal.agent_id)
                    .ok_or_else(|| pmcp::Error::internal("เอเจนต์ที่ถูกเลือกไม่อยู่ในรีจิสทรี"))?
                    .clone();

                (config, Some(proposal))
            } else {
                // เลือกโดยเราเตอร์ทันที
                let agent_configs: Vec<&AgentConfig> = agent_states.iter().map(|s| &s.config).collect();
                let config = self.router.select_agent(&args.task_type, &args.prompt, &agent_configs)
                    .map_err(|e| pmcp::Error::internal(e.to_string()))?
                    .clone();
                (config, None)
            }
        };

        let agent_id = agent_config.id.clone();
        // กำหนดสถานะเริ่มต้น
        let status = if args.interactive {
            TaskStatus::AwaitingApproval
        } else {
            TaskStatus::Pending
        };

        // ปลดล็อกการอ่านและล็อกเขียนเพื่อลงทะเบียน task
        drop(registry);
        let mut registry = self.agent_registry.write().await;
        registry.register_task(TaskInfo {
            task_id: task_id.clone(),
            agent_id: agent_id.clone(),
            task_type: args.task_type.clone(),
            status: status.clone(),
            proposal: proposal.clone(),
            prompt: args.prompt.clone(),
            context: args.context.clone(),
        });
        drop(registry);

        // หากอยู่ในสถานะรออนุมัติ ให้ส่งคืนทันที
        if status == TaskStatus::AwaitingApproval {
            return Ok(DelegateTaskOutput {
                task_id,
                agent_id,
                status: "awaiting_approval".to_string(),
                result: None,
                proposal,
            });
        }

        // ตรวจสอบ rate limit ก่อนดำเนินการ
        let mut rate_limiter = self.rate_limiter.write().await;
        if !rate_limiter.check_and_increment(&agent_id, &agent_config.rate_limit).await {
            return Err(pmcp::Error::internal(
                format!("เกินขีดจำกัดอัตราการเรียกสำหรับเอเจนต์: {}", agent_id)
            ));
        }
        drop(rate_limiter);

        // ดำเนินงานตามโหมด (background หรือ synchronous)
        if args.background {
            // สร้าง task พื้นหลัง
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
            // รันแบบ synchronous รอผลลัพธ์
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

    /// อนุมัติและดำเนินการงานที่รอการอนุมัติ
    ///
    /// # Arguments
    /// * `task_id` - รหัสงานที่ต้องการอนุมัติ
    /// * `confirmed_agent_id` - เอเจนต์ที่ผู้ใช้ยืนยัน (หากไม่ระบุใช้ตามข้อเสนอ)
    pub async fn approve_task(&self, task_id: String, confirmed_agent_id: Option<String>) -> pmcp::Result<DelegateTaskOutput> {
        let mut registry = self.agent_registry.write().await;
        let mut task = registry.get_task(&task_id)
            .ok_or_else(|| pmcp::Error::validation(format!("ไม่พบงาน: {}", task_id)))?
            .clone();

        // ตรวจสอบว่างานอยู่ในสถานะรออนุมัติ
        if task.status != TaskStatus::AwaitingApproval {
            return Err(pmcp::Error::validation(format!("งาน {} ไม่อยู่ในสถานะรออนุมัติ", task_id)));
        }

        // ใช้เอเจนต์ที่ยืนยันหรือจากข้อเสนอเดิม
        let agent_id = confirmed_agent_id.unwrap_or(task.agent_id.clone());
        let agent_config = registry.get_agent(&agent_id)
            .ok_or_else(|| pmcp::Error::validation(format!("ไม่พบเอเจนต์: {}", agent_id)))?
            .clone();

        // ตรวจสอบ rate limit สำหรับเอเจนต์ที่ได้รับการยืนยันก่อนเปลี่ยนสถานะ
        drop(registry);
        let mut rate_limiter = self.rate_limiter.write().await;
        if !rate_limiter.check_and_increment(&agent_id, &agent_config.rate_limit).await {
            return Err(pmcp::Error::internal(
                format!("เกินขีดจำกัดอัตราการเรียกสำหรับเอเจนต์: {}", agent_id)
            ));
        }
        drop(rate_limiter);

        // อัปเดตงานด้วยเอเจนต์ที่ยืนยันและสถานะ
        let mut registry = self.agent_registry.write().await;
        task.agent_id = agent_id.clone();
        task.status = TaskStatus::Pending;
        registry.register_task(task.clone());
        drop(registry);

        // ดำเนินการงาน (ปัจจุบันรองรับเฉพาะ synchronous สำหรับการอนุมัติ)
        let result = self.execute_agent_task(
            task_id.clone(),
            agent_config,
            task.prompt,
            task.context,
        ).await.map_err(|e| pmcp::Error::internal(e.to_string()))?;

        Ok(DelegateTaskOutput {
            task_id,
            agent_id,
            status: "completed".to_string(),
            result: Some(result),
            proposal: None,
        })
    }

    /// ดำเนินการงานบนเอเจนต์ที่ระบุโดยใช้โปรโตคอล ACP
    ///
    /// # Arguments
    /// * `task_id` - รหัสงาน
    /// * `agent` - คอนฟิกของเอเจนต์
    /// * `prompt` - ข้อความพร้อมท์
    /// * `context` - บริบทเพิ่มเติม (optional)
    async fn execute_agent_task(
        &self,
        task_id: String,
        agent: AgentConfig,
        prompt: String,
        context: Option<Value>,
    ) -> Result<String> {
        // อัปเดตสถานะเป็นกำลังทำงาน
        let mut registry = self.agent_registry.write().await;
        registry.update_task_status(&task_id, TaskStatus::Running)?;
        drop(registry);

        tracing::info!("กำลังดำเนินงาน {} บนเอเจนต์ {}", task_id, agent.id);

        // เลือกวิธีการดำเนินการตามประเภทของเอเจนต์
        let result = match agent.agent_type.as_str() {
            "cli" => self.execute_cli_agent(&agent, &prompt, context).await,
            "gemini-extension" => self.execute_gemini_extension(&agent, &prompt).await,
            "internal" => {
                if agent.command.as_deref() == Some("pmat-internal") {
                    self.execute_internal_pmat_agent(&prompt).await
                } else {
                    bail!("ไม่รองรับ internal agent: {:?}", agent.command)
                }
            },
            _ => bail!("ไม่รองรับประเภทเอเจนต์: {}", agent.agent_type),
        };

        // อัปเดตสถานะสุดท้าย
        let mut registry = self.agent_registry.write().await;
        match &result {
            Ok(_) => registry.update_task_status(&task_id, TaskStatus::Completed)?,
            Err(_) => registry.update_task_status(&task_id, TaskStatus::Failed)?,
        }

        result
    }

    /// ดำเนินการ internal PMAT agent (เมื่อเปิดใช้งานฟีเจอร์ bundle-pmat)
    #[cfg(feature = "bundle-pmat")]
    async fn execute_internal_pmat_agent(&self, prompt: &str) -> Result<String> {
        tracing::debug!("กำลังเรียกใช้ bundled PMAT agent ด้วยพร้อมท์: {}", prompt);
        let result = run_context_analysis(prompt).await
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        Ok(result)
    }

    /// fallback เมื่อไม่ได้เปิดใช้งานฟีเจอร์ bundle-pmat
    #[cfg(not(feature = "bundle-pmat"))]
    async fn execute_internal_pmat_agent(&self, _prompt: &str) -> Result<String> {
        bail!("มีการเรียกใช้ internal PMAT agent แต่ไม่ได้เปิดใช้งานฟีเจอร์ 'bundle-pmat' กรุณาคอมไพล์ด้วย --features bundle-pmat หรือใช้ pmat เวอร์ชัน CLI")
    }

    /// ดำเนินการ CLI agent ผ่านโปรโตคอล ACP (JSON-RPC)
    ///
    /// ส่งคำขอแบบ line-delimited JSON ไปยัง stdin และอ่านผลลัพธ์จาก stdout
    async fn execute_cli_agent(
        &self,
        agent: &AgentConfig,
        prompt: &str,
        context: Option<Value>,
    ) -> Result<String> {
        let command = agent.command.as_ref()
            .context("CLI agent ต้องการ command")?;

        tracing::debug!("กำลัง spawn กระบวนการ: {} {:?}", command, agent.args);

        // สร้าง child process พร้อม pipe สำหรับ stdin/stdout
        let mut child = Command::new(command)
            .args(agent.args.as_deref().unwrap_or(&[]))
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .context("ไม่สามารถ spawn กระบวนการของเอเจนต์ได้")?;

        let stdin = child.stdin.take().context("ไม่สามารถเข้าถึง stdin")?;
        let stdout = child.stdout.take().context("ไม่สามารถเข้าถึง stdout")?;

        // สร้างคำขอ JSON-RPC ตาม ACP
        let acp_request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "context",
            "params": {
                "prompt": prompt,
                "context": context,
                "format": "llm-optimized"
            }
        });

        // ส่งคำขอเป็นบรรทัดเดียว (line-delimited)
        let request_line = serde_json::to_string(&acp_request)? + "\n";
        self.write_to_agent(stdin, &request_line).await?;

        // อ่านผลลัพธ์
        let response = self.read_from_agent(stdout).await?;

        // รอให้ process จบและตรวจสอบ exit status
        let status = child.wait().await?;
        if !status.success() {
            bail!("กระบวนการของเอเจนต์จบด้วยข้อผิดพลาด: {}", status);
        }

        Ok(response)
    }

    /// เขียนข้อมูลไปยัง stdin ของเอเจนต์
    async fn write_to_agent(&self, mut stdin: ChildStdin, data: &str) -> Result<()> {
        stdin.write_all(data.as_bytes()).await?;
        stdin.flush().await?;
        Ok(())
    }

    /// อ่านบรรทัดจาก stdout ของเอเจนต์และ parse เป็น JSON-RPC response
    async fn read_from_agent(&self, stdout: ChildStdout) -> Result<String> {
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();

        // อ่านหนึ่งบรรทัด (line-delimited JSON)
        reader.read_line(&mut line).await?;

        let response: Value = serde_json::from_str(&line)
            .context("ไม่สามารถ parse การตอบกลับของเอเจนต์")?;

        // ตรวจสอบ field "result" หรือ "error"
        if let Some(result) = response.get("result") {
            if result.is_string() {
                Ok(result.as_str().unwrap().to_string())
            } else {
                Ok(result.to_string())
            }
        } else if let Some(error) = response.get("error") {
            bail!("เอเจนต์ส่งค่าผิดพลาด: {}", error);
        } else {
            bail!("การตอบกลับ JSON-RPC ไม่ถูกต้อง");
        }
    }

    /// ดำเนินการ Gemini extension agent (ยังไม่ได้ดำเนินการ)
    async fn execute_gemini_extension(
        &self,
        agent: &AgentConfig,
        _prompt: &str,
    ) -> Result<String> {
        let extension_name = agent.extension_name.as_ref()
            .context("Extension agent ต้องการ extension_name")?;

        tracing::info!("กำลังเรียกใช้ Gemini extension: {}", extension_name);
        bail!("ยังไม่รองรับ Gemini extension ในขณะนี้")
    }

    /// สร้าง clone ของ executor สำหรับใช้ใน background task
    ///
    /// หมายเหตุ: เราเตอร์ถูกสร้างใหม่ด้วยค่าว่าง (RoutingConfig::default())
    /// เนื่องจาก background task ไม่จำเป็นต้องใช้เราเตอร์อีกหลังจากเลือกเอเจนต์แล้ว
    fn clone_for_background(&self) -> Self {
        Self {
            agent_registry: self.agent_registry.clone(),
            rate_limiter: self.rate_limiter.clone(),
            // สร้างเราเตอร์เปล่าสำหรับ background executor
            router: AgentRouter::new(RoutingConfig {
                rules: vec![],
                tier: RoutingTier::Default,
            }),
        }
    }
}

// -----------------------------------------------------------------------------
// Tests
// -----------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    // สามารถเพิ่ม tests ได้ในภายหลัง
}