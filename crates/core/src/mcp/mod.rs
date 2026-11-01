pub mod protocol;
use crate::agents::{AgentExecutor, AgentRegistry};
use crate::config::Config;
use crate::rate_limit::RateLimitTracker;
use anyhow::Result;
use pmcp::{RequestHandlerExtra, ServerBuilder, TypedTool};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct Orchestrator {
    #[allow(dead_code)]
    config: Config,
    agent_registry: Arc<RwLock<AgentRegistry>>,
    #[allow(dead_code)]
    rate_limiter: Arc<RwLock<RateLimitTracker>>,
    executor: Arc<AgentExecutor>,
}

/// Arguments for delegating a task to a sub-agent
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct DelegateTaskArgs {
    #[schemars(description = "Type of task (e.g., 'code-generation', 'background-task')")]
    pub task_type: String,

    #[schemars(description = "Prompt/instruction for the agent")]
    pub prompt: String,

    #[schemars(description = "Optional agent ID to use (auto-selects if not provided)")]
    pub agent_id: Option<String>,

    #[schemars(description = "Run as background task")]
    #[serde(default)]
    pub background: bool,

    #[schemars(description = "Additional context as JSON")]
    pub context: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct DelegateTaskOutput {
    pub task_id: String,
    pub agent_id: String,
    pub status: String,
    pub result: Option<String>,
}

/// Arguments for querying agent status
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct AgentStatusArgs {
    #[schemars(description = "Optional task ID to query specific task")]
    pub task_id: Option<String>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct AgentStatusOutput {
    pub active_tasks: usize,
    pub available_agents: Vec<String>,
    pub task_info: Option<serde_json::Value>,
}

impl Orchestrator {
    pub async fn new(config: Config) -> Result<Self> {
        // โค้ดส่วนนี้ยังคงทำงานได้ถูกต้อง
        // `config.agents` จะมี pmat-internal agent รวมอยู่ด้วย
        // ถ้าเราแก้ไข `config.rs` ให้เพิ่มมันเข้าไปเมื่อเปิดฟีเจอร์ `bundle-pmat`
        let agent_registry = Arc::new(RwLock::new(AgentRegistry::new(config.agents.clone())));

        let rate_limiter = Arc::new(RwLock::new(RateLimitTracker::new(
            config.rate_limiting.clone(),
        )));

        let executor = Arc::new(AgentExecutor::new(
            agent_registry.clone(),
            rate_limiter.clone(),
            config.routing.clone(),
        ));

        Ok(Self {
            config,
            agent_registry,
            rate_limiter,
            executor,
        })
    }

    pub async fn run_stdio(self) -> Result<()> {
        let executor = self.executor.clone();
        let agent_registry = self.agent_registry.clone();

        // Build MCP server with typed tools
        let server = ServerBuilder::new()
            .name("bl1nk-agents-manager")
            .version("0.1.0")
            // Tool: Delegate task to sub-agent
            .tool(
                "delegate_task",
                TypedTool::new("delegate_task", {
                    let executor = executor.clone();
                    move |args: DelegateTaskArgs, _extra: RequestHandlerExtra| {
                        let executor = executor.clone();
                        Box::pin(async move {
                            let output = executor.delegate_task(args).await?;
                            Ok(serde_json::to_value(output)?)
                        })
                    }
                })
                .with_description("Delegate a task to an appropriate sub-agent"),
            )
            // Tool: Query agent status
            .tool(
                "agent_status",
                TypedTool::new("agent_status", {
                    let agent_registry = agent_registry.clone();
                    move |args: AgentStatusArgs, _extra: RequestHandlerExtra| {
                        let agent_registry = agent_registry.clone();
                        Box::pin(async move {
                            let output = query_agent_status(agent_registry, args).await?;
                            Ok(serde_json::to_value(output)?)
                        })
                    }
                })
                .with_description("Get status of agents and running tasks"),
            )
            .build()?;

        // Run the MCP server on stdio
        server.run_stdio().await?;
        Ok(())
    }
}

async fn query_agent_status(
    registry: Arc<RwLock<AgentRegistry>>,
    args: AgentStatusArgs,
) -> pmcp::Result<AgentStatusOutput> {
    let registry = registry.read().await;

    Ok(AgentStatusOutput {
        active_tasks: registry.active_task_count(),
        available_agents: registry.list_agent_ids(),
        task_info: args.task_id.map(|id| {
            serde_json::json!({
                "task_id": id,
                "status": "pending"
            })
        }),
    })
}
