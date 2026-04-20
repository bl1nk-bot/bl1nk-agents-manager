//! Unified Registry Schema Types (v1.7.5.1)
//!
//! กำหนด types สำหรับ Registry และ Agent Metadata
//! รองรับโครงสร้างแบบแยกส่วน (Split Structure) และมาตรฐาน Universal Tools

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Registry & Agent Types
// ============================================================================

/// Registry หลัก เก็บข้อมูลเอเจนต์ทางเทคนิคทั้งหมด
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Registry {
    pub version: String,
    #[serde(default)]
    pub last_updated: Option<String>,
    pub agents: Vec<AgentJsonEntry>,
}

/// ข้อมูลทางเทคนิคของเอเจนต์ที่เก็บใน agents.json
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AgentJsonEntry {
    pub name: String,
    pub version: String,
    pub source: AgentSource,
    #[serde(rename = "type")]
    pub agent_type: String,
    pub tier: u8,
    pub priority: u16,
    pub policies: AgentPoliciesJson,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub color: Option<String>,
}

/// แหล่งที่มาของเอเจนต์ (Source) - รองรับ Builtin, Git, Local
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgentSource {
    Builtin { path: String },
    Git { url: String, ref_name: Option<String> },
    Local { path: String },
    Url { url: String },
}

/// กฎการควบคุมสิทธิ์แบบ Nested Map สำหรับเครื่องมือทั้งหมด
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AgentPoliciesJson {
    pub tools: HashMap<String, String>,
}

// ... (Rest of Monitoring Layer remains same for compatibility)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HumanAction {
    Approve,
    Reject,
    Modify,
    Ignore,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "layer_type", rename_all = "snake_case")]
pub enum MonitoringLayer {
    Human {
        user_id: String,
        action: HumanAction,
        response_time: std::time::Duration,
        modification_details: Option<String>,
    },
    Model {
        agent_id: String,
        claim: String,
        actual_result: String,
        self_assessment: String,
        objective_assessment: String,
        confidence: f32,
    },
    Tool {
        tool_name: String,
        input: String,
        output: String,
        success: bool,
        error_type: Option<String>,
        execution_time: std::time::Duration,
        retry_count: usize,
    },
    Input {
        source: String,
        content: String,
        validated: bool,
        validation_errors: Vec<String>,
    },
    Output {
        destination: String,
        content: String,
        expected_format: String,
        format_compliant: bool,
        quality_score: f32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MonitoringRecord {
    pub id: String,
    pub timestamp: String,
    pub layer: MonitoringLayer,
    pub task_id: Option<String>,
    pub session_id: String,
    pub expected: Option<String>,
    pub actual: String,
    pub delta: Option<String>,
    pub quality_score: f32,
    pub evidence: Vec<EvidenceRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Trend {
    Improving,
    Stable,
    Degrading,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "evidence_type", rename_all = "snake_case")]
pub enum EvidenceRecord {
    TestResults {
        command: String,
        output: String,
        pass_count: usize,
        fail_count: usize,
        coverage_percentage: Option<f32>,
    },
    FileChanges {
        files: Vec<String>,
        diff_summary: String,
        lines_added: usize,
        lines_removed: usize,
    },
    CommandOutput {
        command: String,
        stdout: String,
        stderr: String,
        exit_code: i32,
        execution_time: std::time::Duration,
    },
    DecisionLog {
        decision: String,
        reasoning: String,
        alternatives_considered: Vec<String>,
        why_this_choice: String,
        risk_assessment: Option<String>,
    },
    ProgressMetrics {
        metric_name: String,
        before_value: f32,
        after_value: f32,
        improvement_percentage: f32,
        trend: Trend,
    },
    AlternativePath {
        original_path: String,
        alternative_path: String,
        why_alternative: String,
        success_rate_comparison: Option<(f32, f32)>,
    },
}
