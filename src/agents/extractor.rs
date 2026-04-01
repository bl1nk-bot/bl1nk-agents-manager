use crate::config::{AgentConfig, RoutingConfig, RoutingTier};
use crate::agents::{AgentRegistry, AgentRouter, register::{TaskInfo, TaskStatus}};
use crate::mcp::{DelegateTaskArgs, DelegateTaskOutput};
use crate::rate_limit::RateLimitTracker;
use anyhow::{Result, Context, bail};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::process::{Command, ChildStdin, ChildStdout};
use tokio::io::{AsyncWriteExt, AsyncBufReadExt, BufReader};
use serde_json::Value;
use uuid::Uuid;

// Conditional Import for bundled pmat
#[cfg(feature = "bundle-pmat")]
use pmat_core::run_context_analysis; // สมมติว่า pmat-core มีฟังก์ชันนี้


pub struct AgentExecutor {
    agent_registry: Arc<RwLock<AgentRegistry>>,
    rate_limiter: Arc<RwLock<RateLimitTracker>>,
    router: AgentRouter,
}

impl AgentExecutor {
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

    /// Delegate a task to an appropriate sub-agent using ACP
    pub async fn delegate_task(&self, args: DelegateTaskArgs) -> pmcp::Result<DelegateTaskOutput> {
        // Generate task ID
        let task_id = Uuid::new_v4().to_string();

        // Select agent or Generate Proposal
        let registry = self.agent_registry.read().await;

        let (agent_config, proposal) = if let Some(agent_id) = &args.agent_id {
            let config = registry.get_agent(agent_id)
                .ok_or_else(|| pmcp::Error::validation(format!("Agent not found: {}", agent_id)))?
                .clone();
            (config, None)
        } else {
            // Auto-select based on task_type
            let agent_states = registry.get_agents_sorted();

            if args.interactive {
                let proposal = self.router.create_proposal(&args.task_type, &args.prompt, &agent_states)
                    .map_err(|e| pmcp::Error::internal(e.to_string()))?;

                let config = registry.get_agent(&proposal.agent_id)
                    .ok_or_else(|| pmcp::Error::internal("Selected agent not found in registry"))?
                    .clone();

                (config, Some(proposal))
            } else {
                let agent_configs: Vec<&AgentConfig> = agent_states.iter().map(|s| &s.config).collect();
                let config = self.router.select_agent(&args.task_type, &args.prompt, &agent_configs)
                    .map_err(|e| pmcp::Error::internal(e.to_string()))?
                    .clone();
                (config, None)
            }
        };

        let agent_id = agent_config.id.clone();
        let status = if args.interactive {
            TaskStatus::AwaitingApproval
        } else {
            TaskStatus::Pending
        };

        // Register task
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

        if status == TaskStatus::AwaitingApproval {
            return Ok(DelegateTaskOutput {
                task_id,
                agent_id,
                status: "awaiting_approval".to_string(),
                result: None,
                proposal,
            });
        }

        // Check rate limit
        let mut rate_limiter = self.rate_limiter.write().await;
        if !rate_limiter.check_and_increment(&agent_id, &agent_config.rate_limit).await {
            return Err(pmcp::Error::internal(
                format!("Rate limit exceeded for agent: {}", agent_id)
            ));
        }
        drop(rate_limiter);

        // Execute task
        if args.background {
            // Spawn background task
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
                    tracing::error!("Background task failed: {}", e);
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
            // Execute synchronously
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

    /// Approve and execute a task that is awaiting approval
    pub async fn approve_task(&self, task_id: String, confirmed_agent_id: Option<String>) -> pmcp::Result<DelegateTaskOutput> {
        let mut registry = self.agent_registry.write().await;
        let mut task = registry.get_task(&task_id)
            .ok_or_else(|| pmcp::Error::validation(format!("Task not found: {}", task_id)))?
            .clone();

        if task.status != TaskStatus::AwaitingApproval {
            return Err(pmcp::Error::validation(format!("Task {} is not awaiting approval", task_id)));
        }

        let agent_id = confirmed_agent_id.unwrap_or(task.agent_id.clone());
        let agent_config = registry.get_agent(&agent_id)
            .ok_or_else(|| pmcp::Error::validation(format!("Agent not found: {}", agent_id)))?
            .clone();

        // Check rate limit for confirmed agent before changing task state
        drop(registry);
        let mut rate_limiter = self.rate_limiter.write().await;
        if !rate_limiter.check_and_increment(&agent_id, &agent_config.rate_limit).await {
            return Err(pmcp::Error::internal(
                format!("Rate limit exceeded for agent: {}", agent_id)
            ));
        }
        drop(rate_limiter);

        // Update task with confirmed agent and status
        let mut registry = self.agent_registry.write().await;
        task.agent_id = agent_id.clone();
        task.status = TaskStatus::Pending;
        registry.register_task(task.clone());
        drop(registry);

        // Execute task (currently only synchronous approval supported for simplicity)
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

    /// Execute task on a specific agent using ACP protocol
    async fn execute_agent_task(
        &self,
        task_id: String,
        agent: AgentConfig,
        prompt: String,
        context: Option<Value>,
    ) -> Result<String> {
        // Update status to running
        let mut registry = self.agent_registry.write().await;
        registry.update_task_status(&task_id, TaskStatus::Running)?;
        drop(registry);

        tracing::info!("Executing task {} on agent {}", task_id, agent.id);

        let result = match agent.agent_type.as_str() {
            "cli" => self.execute_cli_agent(&agent, &prompt, context).await,
            "gemini-extension" => self.execute_gemini_extension(&agent, &prompt).await,
            "internal" => {
                if agent.command.as_deref() == Some("pmat-internal") {
                    self.execute_internal_pmat_agent(&prompt).await
                } else {
                    bail!("Unsupported internal agent: {:?}", agent.command)
                }
            },
            _ => bail!("Unsupported agent type: {}", agent.agent_type),
        };

        // Update final status
        let mut registry = self.agent_registry.write().await;
        match &result {
            Ok(_) => registry.update_task_status(&task_id, TaskStatus::Completed)?,
            Err(_) => registry.update_task_status(&task_id, TaskStatus::Failed)?,
        }

        result
    }

    #[cfg(feature = "bundle-pmat")]
    async fn execute_internal_pmat_agent(&self, prompt: &str) -> Result<String> {
        tracing::debug!("Executing bundled PMAT agent with prompt: {}", prompt);
        let result = run_context_analysis(prompt).await.map_err(|e| anyhow::anyhow!(e.to_string()))?;
        Ok(result)
    }

    #[cfg(not(feature = "bundle-pmat"))]
    async fn execute_internal_pmat_agent(&self, _prompt: &str) -> Result<String> {
        bail!("Internal PMAT agent called, but the 'bundle-pmat' feature is not enabled. Please compile with --features bundle-pmat or use the CLI version of pmat.")
    }

    async fn execute_cli_agent(
        &self,
        agent: &AgentConfig,
        prompt: &str,
        context: Option<Value>,
    ) -> Result<String> {
        let command = agent.command.as_ref()
            .context("CLI agent requires command")?;

        tracing::debug!("Spawning process: {} {:?}", command, agent.args);

        let mut child = Command::new(command)
            .args(agent.args.as_deref().unwrap_or(&[]))
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .context("Failed to spawn agent process")?;

        let stdin = child.stdin.take().context("Failed to get stdin")?;
        let stdout = child.stdout.take().context("Failed to get stdout")?;

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

        let request_line = serde_json::to_string(&acp_request)? + "\n";
        self.write_to_agent(stdin, &request_line).await?;

        let response = self.read_from_agent(stdout).await?;

        let status = child.wait().await?;
        if !status.success() {
            bail!("Agent process exited with error: {}", status);
        }

        Ok(response)
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

        let response: Value = serde_json::from_str(&line)
            .context("Failed to parse agent response")?;

        if let Some(result) = response.get("result") {
            if result.is_string() {
                Ok(result.as_str().unwrap().to_string())
            } else {
                Ok(result.to_string())
            }
        } else if let Some(error) = response.get("error") {
            bail!("Agent returned error: {}", error);
        } else {
            bail!("Invalid JSON-RPC response");
        }
    }

    async fn execute_gemini_extension(
        &self,
        agent: &AgentConfig,
        prompt: &str,
    ) -> Result<String> {
        let extension_name = agent.extension_name.as_ref()
            .context("Extension agent requires extension_name")?;

        tracing::info!("Calling Gemini extension: {}", extension_name);
        bail!("Gemini extension support not yet implemented")
    }

    fn clone_for_background(&self) -> Self {
        Self {
            agent_registry: self.agent_registry.clone(),
            rate_limiter: self.rate_limiter.clone(),
            router: AgentRouter::new(RoutingConfig { rules: vec![], tier: RoutingTier::Default }),
        }
    }
}
