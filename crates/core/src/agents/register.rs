use crate::agents::types::AgentConfig;
use anyhow::{Context, Result};
use std::collections::HashMap;

pub struct AgentRegistry {
    agents: HashMap<String, AgentConfig>,
    active_tasks: HashMap<String, TaskInfo>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskInfo {
    pub task_id: String,
    pub agent_id: String,
    pub task_type: String,
    pub status: TaskStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

impl AgentRegistry {
    pub fn new(agents: Vec<AgentConfig>) -> Self {
        let agents_map = agents
            .into_iter()
            .map(|agent| (agent.id.clone(), agent))
            .collect();

        Self {
            agents: agents_map,
            active_tasks: HashMap::new(),
        }
    }

    /// Get agent by ID
    pub fn get_agent(&self, id: &str) -> Option<&AgentConfig> {
        self.agents.get(id)
    }

    /// Get agents by capability
    pub fn get_agents_by_capability(&self, capability: &str) -> Vec<&AgentConfig> {
        self.agents
            .values()
            .filter(|agent| agent.capabilities.contains(&capability.to_string()))
            .collect()
    }

    /// Get all agent IDs
    pub fn list_agent_ids(&self) -> Vec<String> {
        self.agents.keys().cloned().collect()
    }

    /// Get all agents sorted by priority (higher first)
    pub fn get_agents_by_priority(&self) -> Vec<&AgentConfig> {
        let mut agents: Vec<&AgentConfig> = self.agents.values().collect();
        agents.sort_by(|a, b| b.priority.cmp(&a.priority));
        agents
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
            .filter(|task| matches!(task.status, TaskStatus::Running | TaskStatus::Pending))
            .count()
    }

    /// Remove completed/failed tasks (cleanup)
    pub fn cleanup_finished_tasks(&mut self) {
        self.active_tasks
            .retain(|_, task| matches!(task.status, TaskStatus::Running | TaskStatus::Pending));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::types::RateLimit;

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
                description: None,
                model: None,
                temperature: None,
                max_tokens: None,
                prompt: None,
                color: None,
                permission: None,
                mode: None,
                thinking: None,
                reasoning_effort: None,
                text_verbosity: None,
                skills: None,
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
                description: None,
                model: None,
                temperature: None,
                max_tokens: None,
                prompt: None,
                color: None,
                permission: None,
                mode: None,
                thinking: None,
                reasoning_effort: None,
                text_verbosity: None,
                skills: None,
            },
        ]
    }

    #[test]
    fn test_agent_registry_creation() {
        let agents = create_test_agents();
        let registry = AgentRegistry::new(agents);

        assert_eq!(registry.list_agent_ids().len(), 2);
        assert!(registry.get_agent("cli-agent").is_some());
        assert!(registry.get_agent("internal-pmat").is_some());
    }

    #[test]
    fn test_get_agents_by_priority() {
        let agents = create_test_agents();
        let registry = AgentRegistry::new(agents);
        let sorted_agents = registry.get_agents_by_priority();

        assert_eq!(sorted_agents.len(), 2);
        assert_eq!(sorted_agents[0].id, "internal-pmat");
        assert_eq!(sorted_agents[1].id, "cli-agent");
    }

    #[test]
    fn test_get_agents_by_capability() {
        let agents = create_test_agents();
        let registry = AgentRegistry::new(agents);

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
    fn test_task_management() {
        let agents = create_test_agents();
        let mut registry = AgentRegistry::new(agents);

        let task_info = TaskInfo {
            task_id: "task-123".to_string(),
            agent_id: "internal-pmat".to_string(),
            task_type: "code-analysis".to_string(),
            status: TaskStatus::Pending,
        };

        registry.register_task(task_info.clone());
        assert_eq!(registry.active_task_count(), 1);

        registry
            .update_task_status("task-123", TaskStatus::Running)
            .unwrap();
        let updated_task = registry.active_tasks.get("task-123").unwrap();
        assert_eq!(updated_task.status, TaskStatus::Running);
        assert_eq!(registry.active_task_count(), 1);

        registry
            .update_task_status("task-123", TaskStatus::Completed)
            .unwrap();
        assert_eq!(registry.active_task_count(), 0);

        registry.cleanup_finished_tasks();
        assert!(registry.active_tasks.get("task-123").is_none());
    }
}
