pub mod schema;

use crate::registry::schema::Registry;
use anyhow::{anyhow, Result};
use jsonschema::JSONSchema;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

/// โหลดและคอมไพล์ Capability Schema (v1.7.2)
static CAPABILITY_SCHEMA: Lazy<Option<JSONSchema>> = Lazy::new(|| {
    let schema_path = "config/v1.7/capability-schema.json";
    if let Ok(content) = fs::read_to_string(schema_path) {
        if let Ok(schema_json) = serde_json::from_str(&content) {
            return JSONSchema::compile(&schema_json).ok();
        }
    }
    None
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub term: String,
    pub score: f32,
}

pub struct RegistryService {
    registry: Registry,
}

impl RegistryService {
    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path).map_err(|e| anyhow!("Failed to read registry file: {}", e))?;
        let registry: Registry =
            serde_json::from_str(&content).map_err(|e| anyhow!("Failed to parse registry JSON: {}", e))?;
        Ok(Self { registry })
    }

    pub fn new(registry: Registry) -> Self {
        Self { registry }
    }

    pub fn search_agents(&self, query: &str, fuzzy: bool) -> Vec<SearchResult> {
        if query.trim().is_empty() {
            return Vec::new();
        }
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();

        for agent in &self.registry.agents {
            let mut score: f32 = 0.0;
            if agent.name.to_lowercase() == query_lower {
                score = 1.0;
            } else if fuzzy && agent.name.to_lowercase().contains(&query_lower) {
                score = 0.8;
            }
            for cap in &agent.capabilities {
                if cap.to_lowercase() == query_lower {
                    score = score.max(0.9);
                } else if fuzzy && cap.to_lowercase().contains(&query_lower) {
                    score = score.max(0.7);
                }
            }
            if score > 0.0 {
                results.push(SearchResult {
                    id: agent.name.clone(),
                    term: agent.name.clone(),
                    score,
                });
            }
        }
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results
    }

    pub fn validate_agent_spec(&self, agent_json: &serde_json::Value) -> Result<()> {
        if let Some(validator) = &*CAPABILITY_SCHEMA {
            if let Err(errors) = validator.validate(agent_json) {
                let mut msg = String::from("Agent Capability Validation Failed:\n");
                for error in errors {
                    msg.push_str(&format!("- {}: {}\n", error.instance_path, error));
                }
                anyhow::bail!(msg);
            }
            Ok(())
        } else {
            anyhow::bail!("Capability Schema validator not initialized")
        }
    }

    pub fn registry(&self) -> &Registry {
        &self.registry
    }
}

// ============================================================================
// Policy & Security Layer (Gemini CLI Standard)
// ============================================================================

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum PolicyDecision {
    Allow,
    Deny,
    AskUser,
}

pub struct PolicyEvaluator;
impl PolicyEvaluator {
    pub fn evaluate(agent: &crate::config::AgentConfig, tool_name: &str, _args: &serde_json::Value) -> PolicyDecision {
        let rule = agent.policies.iter().find(|p| p.tool == "*" || p.tool == tool_name);

        let configured_decision = if let Some(r) = rule {
            match r.decision.as_str() {
                "allow" => PolicyDecision::Allow,
                "ask_user" => PolicyDecision::AskUser,
                _ => PolicyDecision::Deny,
            }
        } else {
            PolicyDecision::Deny
        };

        // Security Guard: Tier 2 (Extension) ห้าม Allow เครื่องมืออันตรายอัตโนมัติ
        let dangerous_tools = ["bash", "write", "rm", "shred"];
        if agent.tier <= 2 && dangerous_tools.contains(&tool_name) && configured_decision == PolicyDecision::Allow {
            return PolicyDecision::AskUser;
        }

        configured_decision
    }
}

// ============================================================================
// Behavioral Stats Layer (Dynamic Measurement)
// ============================================================================

use crate::persistence::{Persistence, StorageLocation};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BehavioralStats {
    pub success_count: u32,
    pub total_count: u32,
    pub consecutive_errors: u32,
    pub hidden_error_count: u32,
    pub rule_violation_count: u32,
    pub bypassed_ask_user_count: u32,
    pub user_preference_score: f64, // ขยับตามความพอใจของผู้ใช้
    pub approval_count: u32,
    pub rejection_count: u32,
}

impl Default for BehavioralStats {
    fn default() -> Self {
        Self {
            success_count: 0,
            total_count: 0,
            consecutive_errors: 0,
            hidden_error_count: 0,
            rule_violation_count: 0,
            bypassed_ask_user_count: 0,
            user_preference_score: 0.5,
            approval_count: 0,
            rejection_count: 0,
        }
    }
}

pub struct WeightRegistry {
    pub stats: HashMap<String, BehavioralStats>,
}

impl Default for WeightRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl WeightRegistry {
    pub fn new() -> Self {
        Self { stats: HashMap::new() }
    }
    pub async fn load() -> anyhow::Result<Self> {
        let p = Persistence::new(StorageLocation::Local)?;
        let stats = p
            .load_json(".omg/state/policy_metrics.json")
            .await
            .unwrap_or_else(|_| HashMap::new());
        Ok(Self { stats })
    }
    pub async fn save(&self) -> anyhow::Result<()> {
        let p = Persistence::new(StorageLocation::Local)?;
        p.save_json(".omg/state/policy_metrics.json", &self.stats).await?;
        Ok(())
    }

    /// บันทึกการตัดสินใจของผู้ใช้ (Approved/Rejected) เพื่อปรับคะแนนความพึงพอใจ
    pub fn record_user_interaction(&mut self, id: &str, approved: bool) {
        let s = self.stats.entry(id.to_string()).or_default();
        if approved {
            s.approval_count += 1;
            // เพิ่มคะแนนความพึงพอใจขึ้น 5% (สูงสุดที่ 1.0)
            s.user_preference_score = (s.user_preference_score + 0.05).min(1.0);
        } else {
            s.rejection_count += 1;
            // ลดคะแนนความพึงพอใจลง 10% (ขั้นต่ำที่ 0.0)
            s.user_preference_score = (s.user_preference_score - 0.10).max(0.0);
        }
    }

    pub fn record_result(&mut self, id: &str, success: bool) {
        let s = self.stats.entry(id.to_string()).or_default();
        s.total_count += 1;
        if success {
            s.success_count += 1;
            s.consecutive_errors = 0;
            // งานสำเร็จทำให้ความเชื่อใจเพิ่มขึ้นเล็กน้อย
            s.user_preference_score = (s.user_preference_score + 0.01).min(1.0);
        } else {
            s.consecutive_errors += 1;
        }
    }

    pub fn record_violation(&mut self, id: &str, vtype: ViolationType) {
        let s = self.stats.entry(id.to_string()).or_default();
        match vtype {
            ViolationType::HiddenError => s.hidden_error_count += 1,
            ViolationType::RuleBreak => s.rule_violation_count += 1,
            ViolationType::BypassedAskUser => s.bypassed_ask_user_count += 1,
        }
        // การละเมิดกฎทำให้คะแนนความพึงพอใจดิ่งวูบ
        s.user_preference_score = (s.user_preference_score - 0.20).max(0.0);
    }

    /// คำนวณคะแนนความเชื่อใจ (0.0 - 1.0) อิงตามประวัติการใช้งานจริง
    pub fn get_trust_score(&self, id: &str) -> f64 {
        let s = match self.stats.get(id) {
            Some(v) => v,
            None => return 0.5, // Default score for new agents
        };

        // สูตรคำนวณ: ถ่วงน้ำหนักระหว่าง ความสำเร็จ (30%) และ ความพึงพอใจของผู้ใช้ (70%)
        let success_rate = if s.total_count == 0 {
            0.5
        } else {
            s.success_count as f64 / s.total_count as f64
        };

        let penalty = (s.consecutive_errors as f64 * 0.15) + (s.hidden_error_count as f64 * 0.3);

        (success_rate * 0.3 + s.user_preference_score * 0.7 - penalty).clamp(0.0, 1.0)
    }
}

#[derive(Debug)]
pub enum ViolationType {
    HiddenError,
    RuleBreak,
    BypassedAskUser,
}
