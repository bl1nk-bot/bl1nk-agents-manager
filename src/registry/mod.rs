use bl1nk_keyword_core::{
    KeywordRegistry, KeywordSearch, Validator,
    ValidationError, ValidatorError, SearchResult,
    load_registry, generate_markdown,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// --- Types จาก core library ---
#[cfg(test)]
use bl1nk_keyword_core::schema::{
    Metadata, KeywordGroup, FieldSchema, CustomFieldConfig,
    ValidationConfig, ValidationRules,
};

pub struct RegistryService {
    registry: KeywordRegistry,
    search: KeywordSearch,
    validator: Validator,
}

impl RegistryService {
    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, ValidatorError> {
        let registry = load_registry(&path)?;
        let search = KeywordSearch::new(registry.clone());
        let validator = Validator::new(registry.clone());
        Ok(Self { registry, search, validator })
    }

    pub fn new(registry: KeywordRegistry) -> Self {
        let search = KeywordSearch::new(registry.clone());
        let validator = Validator::new(registry.clone());
        Self { registry, search, validator }
    }

    pub fn search(&self, query: &str, group_id: Option<&str>) -> Vec<SearchResult> {
        if query.trim().is_empty() { return Vec::new(); }
        self.search.search(query, group_id)
    }

    pub fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut all_errors = Vec::new();
        for group in &self.registry.groups {
            for entry in &group.entries {
                if let Err(mut errors) = self.validator.validate_entry(&group.group_id, entry) {
                    all_errors.append(&mut errors);
                }
            }
        }
        if all_errors.is_empty() { Ok(()) } else { Err(all_errors) }
    }

    pub fn validate_entry(&self, group_id: &str, entry: &serde_json::Value) -> Result<(), Vec<ValidationError>> {
        self.validator.validate_entry(group_id, entry)
    }

    pub fn validate_agent_spec(&self, agent_json: &serde_json::Value) -> anyhow::Result<()> {
        let schema_path = ".config/schema-agent.json";
        let schema_content = std::fs::read_to_string(schema_path)?;
        let schema: serde_json::Value = serde_json::from_str(&schema_content)?;
        let compiled = jsonschema::JSONSchema::compile(&schema)
            .map_err(|e| anyhow::anyhow!("Schema compile error: {}", e))?;
        if let Err(errors) = compiled.validate(agent_json) {
            let mut msg = String::from("Agent Spec Validation Failed:\n");
            for error in errors { msg.push_str(&format!("- {}: {}\n", error.instance_path, error)); }
            anyhow::bail!(msg);
        }
        Ok(())
    }

    pub fn registry(&self) -> &KeywordRegistry { &self.registry }
    pub fn to_markdown(&self) -> String { generate_markdown(&self.registry) }

    pub fn analyze_agent_coverage(&self, agents: &[crate::config::AgentConfig]) -> CoverageReport {
        let mut report = CoverageReport::new();
        let mut covered_ids = std::collections::HashSet::new();
        for agent in agents {
            for capability in &agent.capabilities {
                let results = self.search(capability, None);
                for res in results {
                    covered_ids.insert(res.id.clone());
                    report.agent_mapping.entry(res.id.clone()).or_insert_with(Vec::new).push(agent.id.clone());
                }
            }
        }
        for group in &self.registry.groups {
            for entry in &group.entries {
                if let Some(id) = entry.get("id").and_then(|v| v.as_str()) {
                    if !covered_ids.contains(id) { report.missing_coverage.push(id.to_string()); }
                }
            }
        }
        report
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CoverageReport {
    pub agent_mapping: HashMap<String, Vec<String>>,
    pub missing_coverage: Vec<String>,
}

impl CoverageReport {
    fn new() -> Self { Self { agent_mapping: HashMap::new(), missing_coverage: Vec::new() } }
    pub fn summary(&self) -> String {
        format!("📊 Coverage Summary: Covered: {}, Missing: {}, Redundant: {}", 
            self.agent_mapping.len(), self.missing_coverage.len(), 
            self.agent_mapping.values().filter(|v| v.len() > 1).count())
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum PolicyDecision { Allow, Deny, AskUser }

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum PolicyTier { Default = 1, Extension = 2, Workspace = 3, User = 4, Admin = 5 }

impl PolicyTier { pub fn base_score(&self) -> f64 { *self as i32 as f64 } }

pub struct PolicyEvaluator;

impl PolicyEvaluator {
    pub fn calculate_final_score(tier: PolicyTier, priority: u16) -> f64 {
        tier.base_score() + (priority as f64 / 1000.0)
    }

    pub fn evaluate(agent: &crate::config::AgentConfig, tool_name: &str, args: &serde_json::Value) -> PolicyDecision {
        // 1. Implicit Allow
        if tool_name.starts_with("read") || tool_name.starts_with("file_read") || tool_name == "file_list" {
            return PolicyDecision::Allow;
        }
        // 2. Excluded Dangerous Tools
        let dangerous = vec!["rm", "format", "delete_all"];
        if dangerous.contains(&tool_name) && agent.permission < 900 { return PolicyDecision::Deny; }

        // 3. Full Gemini TOML Rule Matching
        if let Some(rules_val) = agent.permission_policy.get("decision_rules").and_then(|v| v.as_array()) {
            let mut rules = rules_val.clone();
            // เรียงลำดับตาม priority (สูง -> ต่ำ)
            rules.sort_by_key(|r| r.get("priority").and_then(|v| v.as_i64()).unwrap_or(0) * -1);

            for rule in rules {
                let rule_tool = rule.get("toolName").and_then(|v| v.as_str()).unwrap_or("");
                if rule_tool == "*" || rule_tool == tool_name {
                    let dec = match rule.get("decision").and_then(|v| v.as_str()).unwrap_or("deny") {
                        "allow" => PolicyDecision::Allow,
                        "ask_user" => PolicyDecision::AskUser,
                        _ => PolicyDecision::Deny,
                    };

                    // --- Condition Matching ---
                    let mut matched = true;

                    // A) Args Pattern (Regex)
                    if let Some(pattern) = rule.get("argsPattern").and_then(|v| v.as_str()) {
                        if let Ok(re) = regex::Regex::new(pattern) {
                            if !re.is_match(&args.to_string()) { matched = false; }
                        }
                    }

                    // B) Command Prefix (สำหรับ bash/shell)
                    if tool_name == "bash" || tool_name == "run_shell_command" {
                        if let Some(prefix) = rule.get("commandPrefix").and_then(|v| v.as_str()) {
                            let cmd = args.get("command").and_then(|v| v.as_str()).unwrap_or("");
                            if !cmd.starts_with(prefix) { matched = false; }
                        }
                        
                        // C) Command Regex
                        if let Some(re_str) = rule.get("commandRegex").and_then(|v| v.as_str()) {
                            let cmd = args.get("command").and_then(|v| v.as_str()).unwrap_or("");
                            if let Ok(re) = regex::Regex::new(re_str) {
                                if !re.is_match(cmd) { matched = false; }
                            }
                        }
                    }

                    if matched {
                        if let Some(reason) = rule.get("reason").and_then(|v| v.as_str()) {
                            tracing::debug!("🛡️ Policy Match ({:?}): {}", dec, reason);
                        }
                        return dec;
                    }
                }
            }
        }
        if tool_name == "bash" || tool_name == "write" { return PolicyDecision::AskUser; }
        PolicyDecision::Allow
    }
}

use crate::persistence::{Persistence, StorageLocation};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BehavioralStats {
    pub success_count: u32,
    pub total_count: u32,
    pub consecutive_errors: u32,
    pub hidden_error_count: u32,
    pub rule_violation_count: u32,
    pub bypassed_ask_user_count: u32,
    pub user_preference_score: f64, // ค่าที่ผู้ใช้กำหนด (0.0 - 1.0)
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
        }
    }
}

pub struct WeightRegistry { 
    pub stats: HashMap<String, BehavioralStats> 
}

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

    /// บันทึกผลงานปกติ
    pub fn record_result(&mut self, agent_id: &str, success: bool) {
        let s = self.stats.entry(agent_id.to_string()).or_default();
        s.total_count += 1;
        if success {
            s.success_count += 1;
            s.consecutive_errors = 0;
        } else {
            s.consecutive_errors += 1;
        }
    }

    /// บันทึกความผิดลหุโทษ (Penalties)
    pub fn record_violation(&mut self, agent_id: &str, violation_type: ViolationType) {
        let s = self.stats.entry(agent_id.to_string()).or_default();
        match violation_type {
            ViolationType::HiddenError => s.hidden_error_count += 1,
            ViolationType::RuleBreak => s.rule_violation_count += 1,
            ViolationType::BypassedAskUser => s.bypassed_ask_user_count += 1,
        }
    }

    /// คำนวณ Trust Score (0.0 - 1.0) โดยหักลบคะแนนความผิด
    pub fn get_trust_score(&self, agent_id: &str) -> f64 {
        let s = match self.stats.get(agent_id) {
            Some(val) => val,
            None => return 0.5, // Neutral start
        };

        let base_trust = if s.total_count == 0 { 0.5 } else { s.success_count as f64 / s.total_count as f64 };
        
        // บทลงโทษ (Deductions)
        let penalty = (s.consecutive_errors as f64 * 0.1) + 
                      (s.hidden_error_count as f64 * 0.2) + 
                      (s.rule_violation_count as f64 * 0.15) + 
                      (s.bypassed_ask_user_count as f64 * 0.1);

        // รวมกับความชอบของผู้ใช้ (User Preference)
        let score = (base_trust * 0.4) + (s.user_preference_score * 0.6) - penalty;
        
        score.clamp(0.0, 1.0)
    }
}

#[derive(Debug)]
pub enum ViolationType {
    HiddenError,      // เรียก tool ผิดซ้ำแต่ไม่รายงาน
    RuleBreak,        // ไม่ทำตามกฎโครงการ
    BypassedAskUser,  // ข้ามขั้นตอนยืนยันกับผู้ใช้
}
