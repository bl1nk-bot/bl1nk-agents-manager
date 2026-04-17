pub mod schema;

use crate::registry::schema::{Registry, AgentJsonEntry};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use anyhow::{anyhow, Result};
use once_cell::sync::Lazy;
use jsonschema::JSONSchema;

/// โหลดและคอมไพล์ Agent Schema เพียงครั้งเดียวเพื่อประสิทธิภาพ
static AGENT_SCHEMA: Lazy<Option<JSONSchema>> = Lazy::new(|| {
    let schema_path = ".config/schema-agent.json";
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
        let content = fs::read_to_string(path)
            .map_err(|e| anyhow!("Failed to read registry file: {}", e))?;
        let registry: Registry = serde_json::from_str(&content)
            .map_err(|e| anyhow!("Failed to parse registry JSON: {}", e))?;
        Ok(Self { registry })
    }

    pub fn new(registry: Registry) -> Self {
        Self { registry }
    }

    /// ค้นหาเอเจนต์ที่เหมาะสมจากชื่อหรือความสามารถ (Capabilities)
    /// รองรับการเปรียบเทียบเบื้องต้นเพื่อเตรียมพร้อมสำหรับ Semantic Search
    pub fn search_agents(&self, query: &str, fuzzy: bool) -> Vec<SearchResult> {
        if query.trim().is_empty() { return Vec::new(); }
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();

        for agent in &self.registry.agents {
            let mut score: f32 = 0.0;
            
            // 1. Exact Match on Name
            if agent.name.to_lowercase() == query_lower {
                score = 1.0;
            } 
            // 2. Fuzzy Match on Name
            else if fuzzy && agent.name.to_lowercase().contains(&query_lower) {
                score = 0.8;
            }

            // 3. Capability Match (พื้นฐานของ Semantic Search)
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
                    score 
                });
            }
        }
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results
    }

    pub fn validate_agent_spec(&self, agent_json: &serde_json::Value) -> anyhow::Result<()> {
        if let Some(validator) = &*AGENT_SCHEMA {
            if let Err(errors) = validator.validate(agent_json) {
                let mut msg = String::from("Agent Spec Validation Failed:\n");
                for error in errors { msg.push_str(&format!("- {}: {}\n", error.instance_path, error)); }
                anyhow::bail!(msg);
            }
            Ok(())
        } else {
            anyhow::bail!("Agent Schema validator not initialized")
        }
    }

    pub fn registry(&self) -> &Registry { &self.registry }
    
    pub fn search_keywords(&self, query: &str, fuzzy: bool) -> Vec<SearchResult> {
        self.search_agents(query, fuzzy)
    }
}

// ============================================================================
// Policy & Security Layer
// ============================================================================

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum PolicyDecision { Allow, Deny, AskUser }

pub struct PolicyEvaluator;
impl PolicyEvaluator {
    /// ประเมินความปลอดภัยโดยตรวจสอบจากสิทธิ์และเครื่องมือที่เรียกใช้
    pub fn evaluate(agent: &crate::config::AgentConfig, tool_name: &str, _args: &serde_json::Value) -> PolicyDecision {
        // 1. ตรวจสอบเครื่องมืออันตราย (Dangerous Tools)
        // ดึงจาก config หรือใช้ค่า default ที่ปลอดภัย
        let dangerous_tools = vec!["rm", "format", "delete_all", "shred"];
        if dangerous_tools.contains(&tool_name) && agent.permission < 900 {
            return PolicyDecision::Deny;
        }

        // 2. ตรวจสอบกฎจาก Permission Policy
        if let Some(rules) = agent.permission_policy.get("decision_rules").and_then(|v| v.as_array()) {
            for rule in rules {
                let rule_tool = rule.get("toolName").and_then(|v| v.as_str()).unwrap_or("");
                if rule_tool == "*" || rule_tool == tool_name {
                    return match rule.get("decision").and_then(|v| v.as_str()).unwrap_or("deny") {
                        "allow" => PolicyDecision::Allow,
                        "ask_user" => PolicyDecision::AskUser,
                        _ => PolicyDecision::Deny,
                    };
                }
            }
        }

        // 3. Fallback สำหรับเครื่องมือที่มีความเสี่ยงสูง
        if tool_name == "bash" || tool_name == "write" {
            return PolicyDecision::AskUser;
        }

        PolicyDecision::Allow
    }
}

// ============================================================================
// Behavioral Stats Layer
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
    pub user_preference_score: f64,
}

impl Default for BehavioralStats {
    fn default() -> Self {
        Self { success_count: 0, total_count: 0, consecutive_errors: 0, hidden_error_count: 0, 
               rule_violation_count: 0, bypassed_ask_user_count: 0, user_preference_score: 0.5 }
    }
}

pub struct WeightRegistry { pub stats: HashMap<String, BehavioralStats> }
impl WeightRegistry {
    pub fn new() -> Self { Self { stats: HashMap::new() } }
    pub async fn load() -> anyhow::Result<Self> {
        let p = Persistence::new(StorageLocation::Local)?;
        let stats = p.load_json(".omg/state/policy_metrics.json").await.unwrap_or_else(|_| HashMap::new());
        Ok(Self { stats })
    }
    pub async fn save(&self) -> anyhow::Result<()> {
        let p = Persistence::new(StorageLocation::Local)?;
        p.save_json(".omg/state/policy_metrics.json", &self.stats).await?;
        Ok(())
    }
    pub fn record_result(&mut self, id: &str, success: bool) {
        let s = self.stats.entry(id.to_string()).or_default();
        s.total_count += 1;
        if success { s.success_count += 1; s.consecutive_errors = 0; } 
        else { s.consecutive_errors += 1; }
    }
    pub fn record_violation(&mut self, id: &str, vtype: ViolationType) {
        let s = self.stats.entry(id.to_string()).or_default();
        match vtype {
            ViolationType::HiddenError => s.hidden_error_count += 1,
            ViolationType::RuleBreak => s.rule_violation_count += 1,
            ViolationType::BypassedAskUser => s.bypassed_ask_user_count += 1,
        }
    }
    pub fn get_trust_score(&self, id: &str) -> f64 {
        let s = match self.stats.get(id) { Some(v) => v, None => return 0.5 };
        let base = if s.total_count == 0 { 0.5 } else { s.success_count as f64 / s.total_count as f64 };
        let penalty = (s.consecutive_errors as f64 * 0.1) + (s.hidden_error_count as f64 * 0.2);
        (base * 0.4 + s.user_preference_score * 0.6 - penalty).clamp(0.0, 1.0)
    }
}

#[derive(Debug)]
pub enum ViolationType { HiddenError, RuleBreak, BypassedAskUser }
