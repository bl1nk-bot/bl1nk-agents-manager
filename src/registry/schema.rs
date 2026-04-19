//! Unified Registry Schema Types
//!
//! กำหนด types สำหรับ Registry และ Agent Metadata
//! รองรับโครงสร้างแบบแยกส่วน (Split Structure) ระหว่าง .md และ agents.json
//! อัปเดต v1.7.2: รองรับมาตรฐาน Gemini CLI Policy Engine

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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
/// ปรับปรุงตามมาตรฐานสิทธิ์แบบลำดับชั้น (Hierarchical Governance)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AgentJsonEntry {
    pub name: String,
    pub file: String,
    #[serde(rename = "type")]
    pub agent_type: String,

    // มาตรฐาน Gemini CLI Policy Engine
    pub tier: u8,      // 1-5
    pub priority: u16, // 0-999
    pub policies: Vec<PolicyRuleJson>,

    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub color: Option<String>,
}

/// กฎการควบคุมสิทธิ์ตามมาตรฐาน TOML ของ Gemini CLI
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PolicyRuleJson {
    pub tool: String,
    pub decision: String, // "allow" | "deny" | "ask_user"
    #[serde(default)]
    pub modes: Option<Vec<String>>,
}

// ============================================================================
// Monitoring Layer Types (รักษาไว้เพื่อความเข้ากันได้)
// ============================================================================

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
