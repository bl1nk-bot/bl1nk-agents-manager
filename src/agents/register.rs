use crate::config::{AgentConfig, AgentToolPermissions};
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
pub enum TaskStatus { Pending, AwaitingApproval, Running, Completed, Failed }

impl AgentRegistry {
    pub fn new(agents: Vec<AgentConfig>, report: Option<&DiscoveryReport>) -> Self {
        let mut agents_map = HashMap::new();
        for config in agents {
            let availability = Self::calculate_availability(&config, report);
            agents_map.insert(config.id.clone(), AgentState { config, availability });
        }
        Self { agents: agents_map, active_tasks: HashMap::new() }
    }

    fn calculate_availability(config: &AgentConfig, _report: Option<&DiscoveryReport>) -> AgentAvailability {
        // ในเฟสนี้เราเน้นความง่าย ปรับให้พร้อมใช้งานเสมอ (Ready)
        AgentAvailability::Ready
    }

    pub fn get_agent(&self, id: &str) -> Option<&AgentConfig> {
        self.agents.get(id).map(|s| &s.config)
    }

    pub fn get_agent_state(&self, id: &str) -> Option<&AgentState> {
        self.agents.get(id)
    }

    pub fn list_agent_ids(&self) -> Vec<String> {
        self.agents.keys().cloned().collect()
    }

    pub fn get_agents_by_priority(&self) -> Vec<&AgentConfig> {
        let mut states: Vec<&AgentState> = self.agents.values().collect();
        states.sort_by(|a, b| b.config.priority.cmp(&a.config.priority));
        states.into_iter().map(|s| &s.config).collect()
    }

    pub fn get_agents_sorted(&self) -> Vec<&AgentState> {
        let mut states: Vec<&AgentState> = self.agents.values().collect();
        states.sort_by(|a, b| b.config.priority.cmp(&a.config.priority));
        states
    }

    pub fn register_task(&mut self, task: TaskInfo) {
        self.active_tasks.insert(task.task_id.clone(), task);
    }

    pub fn update_task_status(&mut self, task_id: &str, status: TaskStatus) -> Result<()> {
        self.active_tasks.get_mut(task_id).map(|task| task.status = status).context("Task not found")
    }

    pub fn active_task_count(&self) -> usize {
        self.active_tasks.values().filter(|task| matches!(task.status, TaskStatus::Running | TaskStatus::Pending | TaskStatus::AwaitingApproval)).count()
    }

    pub fn get_task(&self, task_id: &str) -> Option<&TaskInfo> {
        self.active_tasks.get(task_id)
    }

    pub fn cleanup_finished_tasks(&mut self) {
        self.active_tasks.retain(|_, task| matches!(task.status, TaskStatus::Running | TaskStatus::Pending | TaskStatus::AwaitingApproval));
    }
}
