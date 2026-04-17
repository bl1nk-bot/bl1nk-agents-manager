//! Unified Registry Schema Types
//!
//! กำหนด types สำหรับ Registry, KeywordEntry, MonitoringRecord, EvidenceRecord
//! และ types ที่เกี่ยวข้องตาม spec ของ Unified Registry, Monitoring & Honesty Layer

use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

// ============================================================================
// Registry & Keyword Types
// ============================================================================

/// Registry หลัก เก็บ keywords ทั้งหมด
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Registry {
    pub version: String,
    pub keywords: Vec<KeywordEntry>,
}

/// Keyword entry พร้อม semantic search และ dynamic weights
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KeywordEntry {
    pub id: String,
    pub term: String,
    pub aliases: Vec<String>,
    pub meanings: Vec<KeywordMeaning>,
    pub expected_evidence: Vec<String>,
    pub base_weight: f32,
    pub calculated_weight: f32,
}

/// ความหมายของ keyword ในบริบทต่างๆ
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KeywordMeaning {
    pub context: String,
    pub definition: String,
    pub related_terms: Vec<String>,
}

// ============================================================================
// Monitoring Layer Types
// ============================================================================

/// การกระทำของผู้ใช้ (Human layer)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HumanAction {
    Approve,
    Reject,
    Modify,
    Ignore,
}

/// Monitoring layer - ตรวจจับทุกชั้น (human, model, tool, input, output)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "layer_type", rename_all = "snake_case")]
pub enum MonitoringLayer {
    /// ผู้ใช้โต้ตอบอย่างไร ตอบสนองต่อข้อเสนอแนะแค่ไหน
    Human {
        user_id: String,
        action: HumanAction,
        response_time: std::time::Duration,
        modification_details: Option<String>,
    },

    /// โมเดลทำอะไร ประกาศตัวเองอย่างไร ผลจริงเป็นอย่างไร
    Model {
        agent_id: String,
        claim: String,
        actual_result: String,
        self_assessment: String,
        objective_assessment: String,
        confidence: f32,
    },

    /// Tool ถูกเรียกอย่างไร สำเร็จหรือล้มเหลว เพราะอะไร
    Tool {
        tool_name: String,
        input: String,
        output: String,
        success: bool,
        error_type: Option<String>,
        execution_time: std::time::Duration,
        retry_count: usize,
    },

    /// Input ที่เข้ามาเป็นอย่างไร ตรงกับที่คาดหรือไม่
    Input {
        source: String,
        content: String,
        validated: bool,
        validation_errors: Vec<String>,
    },

    /// Output ที่ออกไปตรงกับที่คาดหวังหรือไม่
    Output {
        destination: String,
        content: String,
        expected_format: String,
        format_compliant: bool,
        quality_score: f32,
    },
}

/// บันทึก monitoring แต่ละรายการ
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MonitoringRecord {
    pub id: String,
    pub timestamp: String,
    pub layer: MonitoringLayer,
    pub task_id: Option<String>,
    pub session_id: String,

    /// สิ่งที่คาดว่าจะเกิด
    pub expected: Option<String>,
    /// สิ่งที่เกิดขึ้นจริง
    pub actual: String,
    /// ความแตกต่าง
    pub delta: Option<String>,

    /// คะแนนคุณภาพ (0.0-1.0)
    pub quality_score: f32,

    /// หลักฐานประกอบ
    pub evidence: Vec<EvidenceRecord>,
}

// ============================================================================
// Evidence Record Types
// ============================================================================

/// แนวโน้ม (improving, stable, degrading)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Trend {
    Improving,
    Stable,
    Degrading,
}

/// Evidence records หลายประเภท
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "evidence_type", rename_all = "snake_case")]
pub enum EvidenceRecord {
    /// ผลการทดสอบ - พิสูจน์ว่าโค้ดทำงานถูกต้อง
    TestResults {
        command: String,
        output: String,
        pass_count: usize,
        fail_count: usize,
        coverage_percentage: Option<f32>,
    },

    /// ไฟล์ที่เปลี่ยน - พิสูจน์ว่ามีการแก้ไขจริง
    FileChanges {
        files: Vec<String>,
        diff_summary: String,
        lines_added: usize,
        lines_removed: usize,
    },

    /// ผล command - พิสูจน์ว่าคำสั่งทำงานสำเร็จ
    CommandOutput {
        command: String,
        stdout: String,
        stderr: String,
        exit_code: i32,
        execution_time: std::time::Duration,
    },

    /// การตัดสินใจ - บันทึกว่าทำไมถึงเลือกทางนี้
    DecisionLog {
        decision: String,
        reasoning: String,
        alternatives_considered: Vec<String>,
        why_this_choice: String,
        risk_assessment: Option<String>,
    },

    /// ความคืบหน้า - แสดงแนวโน้มว่าดีขึ้นหรือแย่ลง
    ProgressMetrics {
        metric_name: String,
        before_value: f32,
        after_value: f32,
        improvement_percentage: f32,
        trend: Trend,
    },

    /// Alternative Path - แสดงว่ามีทางเลือก ไม่ได้ต้อนไปทางตัน
    AlternativePath {
        original_path: String,
        alternative_path: String,
        why_alternative: String,
        success_rate_comparison: Option<(f32, f32)>,
    },
}

// ============================================================================
// Weight Calculation Types
// ============================================================================

/// ค่าน้ำหนักที่คำนวณได้
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CalculatedWeight {
    pub base_weight: f32,
    pub self_claim_weight: f32,
    pub actual_performance: f32,
    pub tool_success_rate: f32,
    pub error_frequency: f32,
    pub human_approval_rate: f32,
    pub final_weight: f32,
}

// ============================================================================
// Effectiveness Report Types
// ============================================================================

/// การประเมินผลโดยรวม
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Assessment {
    SignificantlyImproved,
    ModeratelyImproved,
    NoChange,
    Degraded,
}

/// เปรียบเทียบ metrics ก่อน-หลัง
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MetricComparison {
    pub name: String,
    pub before: f32,
    pub after: f32,
    pub change_percentage: f32,
    pub trend: Trend,
    pub evidence: Vec<EvidenceRecord>,
}

/// รายงานประสิทธิภาพ
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EffectivenessReport {
    pub period_start: String,
    pub period_end: String,
    pub metrics: Vec<MetricComparison>,
    pub alternative_paths_available: usize,
    pub path_success_rates: Vec<(String, f32)>,
    pub overall_assessment: Assessment,
    pub confidence_level: f32,
}

