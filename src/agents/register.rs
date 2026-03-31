use crate::config::AgentConfig;
use crate::system::discovery::DiscoveryReport;
use crate::agents::router::PlanProposal;
use std::collections::HashMap;
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AgentAvailability {
    Ready,
    MissingTools(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentState {
    pub config: AgentConfig,
    pub availability: AgentAvailability,
}

pub struct AgentRegistry {
    agents: HashMap<String, AgentState>,
    active_tasks: HashMap<String, TaskInfo>,
}

#[derive(Debug, Clone)]
pub struct TaskInfo {
    pub task_id: String,
    pub agent_id: String,
    pub task_type: String,
    pub status: TaskStatus,
    pub proposal: Option<PlanProposal>,
    pub prompt: String,
    pub context: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Pending,
    AwaitingApproval,
    Running,
    Completed,
    Failed,
}

impl AgentRegistry {
    pub fn new(agents: Vec<AgentConfig>, report: Option<&DiscoveryReport>) -> Self {
        let mut agents_map = HashMap::new();
        for config in agents {
            let availability = Self::calculate_availability(&config, report);
            agents_map.insert(config.id.clone(), AgentState { config, availability });
        }

        Self {
            agents: agents_map,
            active_tasks: HashMap::new(),
        }
    }

    pub fn update_availability(&mut self, report: &DiscoveryReport) {
        for state in self.agents.values_mut() {
            state.availability = Self::calculate_availability(&state.config, Some(report));
        }
    }

    fn calculate_availability(config: &AgentConfig, report: Option<&DiscoveryReport>) -> AgentAvailability {
        if config.requires.is_empty() {
            return AgentAvailability::Ready;
        }

        if let Some(report) = report {
            let mut missing = Vec::new();
            for tool in &config.requires {
                if !report.is_tool_available(tool) {
                    missing.push(tool.clone());
                }
            }

            if missing.is_empty() {
                AgentAvailability::Ready
            } else {
                AgentAvailability::MissingTools(missing)
            }
        } else {
            // No report yet, assume missing if tools are required
            AgentAvailability::MissingTools(config.requires.clone())
        }
    }

    /// Get agent by ID
    pub fn get_agent(&self, id: &str) -> Option<&AgentConfig> {
        self.agents.get(id).map(|s| &s.config)
    }

    /// Get agent state by ID
    pub fn get_agent_state(&self, id: &str) -> Option<&AgentState> {
        self.agents.get(id)
    }

    /// Get agents by capability
    pub fn get_agents_by_capability(&self, capability: &str) -> Vec<&AgentConfig> {
        self.agents
            .values()
            .filter(|state| state.config.capabilities.contains(&capability.to_string()))
            .map(|state| &state.config)
            .collect()
    }

    /// Get all agent IDs
    pub fn list_agent_ids(&self) -> Vec<String> {
        self.agents.keys().cloned().collect()
    }

    /// Get all agents sorted by readiness, priority (higher first), and cost (lower first)
    pub fn get_agents_by_priority(&self) -> Vec<&AgentConfig> {
        self.get_agents_sorted()
            .into_iter()
            .map(|s| &s.config)
            .collect()
    }

    /// Get all agent states sorted by readiness, priority, and cost
    pub fn get_agents_sorted(&self) -> Vec<&AgentState> {
        let mut states: Vec<&AgentState> = self.agents.values().collect();

        states.sort_by(|a, b| {
            // 1. Readiness (Ready first)
            let a_ready = matches!(a.availability, AgentAvailability::Ready);
            let b_ready = matches!(b.availability, AgentAvailability::Ready);

            match (a_ready, b_ready) {
                (true, false) => return std::cmp::Ordering::Less,
                (false, true) => return std::cmp::Ordering::Greater,
                _ => {}
            }

            // 2. Priority (higher first)
            match b.config.priority.cmp(&a.config.priority) {
                std::cmp::Ordering::Equal => {
                    // 3. Cost (lower first)
                    a.config.cost.cmp(&b.config.cost)
                }
                ord => ord,
            }
        });

        states
    }

    /// Register a new task
    pub fn register_task(&mut self, task: TaskInfo) {
        self.active_tasks.insert(task.task_id.clone(), task);
    }

    /// Update task status
    pub fn update_task_status(&mut self, task_id: &str, status: TaskStatus) -> Result<()> {
        self.active_tasks
            .get_mut(task_id)
            .map(|task| task.status = status)
            .context("Task not found")
    }

    /// Get active task count
    pub fn active_task_count(&self) -> usize {
        self.active_tasks
            .values()
            .filter(|task| matches!(task.status, TaskStatus::Running | TaskStatus::Pending | TaskStatus::AwaitingApproval))
            .count()
    }

    /// Get task by ID
    pub fn get_task(&self, task_id: &str) -> Option<&TaskInfo> {
        self.active_tasks.get(task_id)
    }

    /// Remove completed/failed tasks (cleanup)
    pub fn cleanup_finished_tasks(&mut self) {
        self.active_tasks.retain(|_, task| {
            matches!(task.status, TaskStatus::Running | TaskStatus::Pending | TaskStatus::AwaitingApproval)
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RateLimit;

    /// Creates a small set of test AgentConfig instances used by unit tests.
    ///
    /// Returns a vector containing two agents:
    /// - a CLI agent with capability `cli-task` and priority 1
    /// - an internal agent with capability `code-analysis` and priority 10
    ///
    /// # Examples
    ///
    /// ```
    /// let agents = create_test_agents();
    /// assert_eq!(agents.len(), 2);
    /// assert!(agents.iter().any(|a| a.id == "cli-agent"));
    /// assert!(agents.iter().any(|a| a.id == "internal-pmat"));
    /// ```
    fn create_test_agents() -> Vec<AgentConfig> {
        vec![
            AgentConfig {
                id: "cli-agent".to_string(),
                name: "CLI Agent".to_string(),
                agent_type: "cli".to_string(),
                command: Some("test-cli".to_string()),
                args: None,
                extension_name: None,
                rate_limit: RateLimit::default(),
                capabilities: vec!["cli-task".to_string()],
                priority: 1,
                enabled: true,
                requires: Vec::new(),
                cost: 0,
            },
            AgentConfig {
                id: "internal-pmat".to_string(),
                name: "Internal PMAT Agent".to_string(),
                agent_type: "internal".to_string(),
                command: Some("pmat-internal".to_string()),
                args: None,
                extension_name: None,
                rate_limit: RateLimit::default(),
                capabilities: vec!["code-analysis".to_string()],
                priority: 10,
                enabled: true,
                requires: Vec::new(),
                cost: 0,
            },
        ]
    }

    #[test]
    fn test_agent_registry_creation() {
        let agents = create_test_agents();
        let registry = AgentRegistry::new(agents, None);

        assert_eq!(registry.list_agent_ids().len(), 2);
        assert!(registry.get_agent("cli-agent").is_some());
        assert!(registry.get_agent("internal-pmat").is_some());
    }

    #[test]
    fn test_get_agents_by_priority() {
        let agents = create_test_agents();
        let registry = AgentRegistry::new(agents, None);
        let sorted_agents = registry.get_agents_by_priority();

        assert_eq!(sorted_agents.len(), 2);
        // ตรวจสอบว่า agent ที่มี priority สูงกว่า (10) มาก่อน
        assert_eq!(sorted_agents[0].id, "internal-pmat");
        assert_eq!(sorted_agents[1].id, "cli-agent");
    }

    #[test]
    fn test_get_agents_by_capability() {
        let agents = create_test_agents();
        let registry = AgentRegistry::new(agents, None);

        let cli_agents = registry.get_agents_by_capability("cli-task");
        assert_eq!(cli_agents.len(), 1);
        assert_eq!(cli_agents[0].id, "cli-agent");

        let analysis_agents = registry.get_agents_by_capability("code-analysis");
        assert_eq!(analysis_agents.len(), 1);
        assert_eq!(analysis_agents[0].id, "internal-pmat");

        let non_existent_agents = registry.get_agents_by_capability("non-existent");
        assert!(non_existent_agents.is_empty());
    }

    #[test]
    fn test_agent_availability_and_sorting() {
        use crate::system::discovery::ToolInfo;
        use chrono::Utc;

        let mut agents = create_test_agents();
        // Add an agent that requires a missing tool
        agents.push(AgentConfig {
            id: "missing-tool-agent".to_string(),
            name: "Missing Tool Agent".to_string(),
            agent_type: "cli".to_string(),
            command: Some("missing".to_string()),
            args: None,
            extension_name: None,
            rate_limit: RateLimit::default(),
            capabilities: vec!["missing-task".to_string()],
            priority: 100, // High priority but missing tool
            enabled: true,
            requires: vec!["non-existent-tool".to_string()],
            cost: 0,
        });

        let report = DiscoveryReport {
            timestamp: Utc::now(),
            ai_clis: vec![ToolInfo {
                name: "test-cli".to_string(),
                version: None,
                available: true,
                path: None,
            }],
            vcs: vec![],
            package_managers: vec![],
        };

        let registry = AgentRegistry::new(agents, Some(&report));

        let sorted = registry.get_agents_by_priority();

        // Even though missing-tool-agent has highest priority (100),
        // it should be last because it's not Ready.
        // Ready agents are internal-pmat (priority 10) and cli-agent (priority 1)
        assert_eq!(sorted[0].id, "internal-pmat");
        assert_eq!(sorted[1].id, "cli-agent");
        assert_eq!(sorted[2].id, "missing-tool-agent");

        let state = registry.get_agent_state("missing-tool-agent").unwrap();
        assert!(matches!(state.availability, AgentAvailability::MissingTools(_)));
    }

    #[test]
    fn test_task_management() {
        let agents = create_test_agents();
        let mut registry = AgentRegistry::new(agents, None);

        let task_info = TaskInfo {
            task_id: "task-123".to_string(),
            agent_id: "internal-pmat".to_string(),
            task_type: "code-analysis".to_string(),
            status: TaskStatus::Pending,
            proposal: None,
            prompt: "test".to_string(),
            context: None,
        };

        registry.register_task(task_info.clone());
        assert_eq!(registry.active_task_count(), 1);

        registry.update_task_status("task-123", TaskStatus::Running).unwrap();
        let updated_task = registry.active_tasks.get("task-123").unwrap();
        assert_eq!(updated_task.status, TaskStatus::Running);
        assert_eq!(registry.active_task_count(), 1);

        registry.update_task_status("task-123", TaskStatus::Completed).unwrap();
        assert_eq!(registry.active_task_count(), 0);

        registry.cleanup_finished_tasks();
        assert!(registry.active_tasks.get("task-123").is_none());
    }
}
