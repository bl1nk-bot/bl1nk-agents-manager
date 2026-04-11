// src/hooks/hook_aggregator.rs
//! ตัวรวบรวมผลลัพธ์จาก Hook หลายตัว (Hook Aggregator)
//!
//! เนื่องจากระบบอาจมี Hook หลายตัวที่ทำงานในเหตุการณ์เดียวกัน
//! HookAggregator จะรวมผลลัพธ์จากทุก Hook เข้าด้วยกันตามกฎเฉพาะของแต่ละประเภทเหตุการณ์
//!
//! กฎการรวมที่สำคัญ:
//! - PreToolUse / PostToolUse: ใช้ OR logic สำหรับการตัดสินใจ (block > allow)
//!   และต่อข้อความเหตุผล (reason) ด้วยขึ้นบรรทัดใหม่
//! - PermissionRequest: behavior "deny" ชนะ "allow", ต่อข้อความ,
//!   updatedInput ใช้ค่าล่าสุด, updatedPermissions ถูกรวมเข้าด้วยกัน
//! - Stop: continue=false ชนะ, stopReason ถูกเก็บจากอันที่หยุด

use std::collections::HashMap;
use std::time::Duration;

// ─────────────────────────────────────────────────────────────────────────────
// ประเภทของเหตุการณ์ Hook (HookEventName)
// ─────────────────────────────────────────────────────────────────────────────

/// ชื่อเหตุการณ์ Hook ที่ระบบรองรับ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HookEventName {
    PreToolUse,
    PostToolUse,
    PostToolUseFailure,
    Stop,
    SubagentStop,
    UserPromptSubmit,
    PermissionRequest,
    // สามารถเพิ่มเหตุการณ์อื่น ๆ ได้ในอนาคต
}

// ─────────────────────────────────────────────────────────────────────────────
// โครงสร้างผลลัพธ์ของ Hook (HookOutput)
// ─────────────────────────────────────────────────────────────────────────────

/// ผลลัพธ์ที่ได้จากการทำงานของ Hook หนึ่งตัว
/// ฟิลด์ทั้งหมดเป็น Option เพราะ Hook แต่ละประเภทอาจคืนค่าต่างกัน
#[derive(Debug, Clone, Default)]
pub struct HookOutput {
    /// การตัดสินใจ: "allow", "block", "deny" ฯลฯ
    pub decision: Option<String>,
    /// เหตุผลประกอบการตัดสินใจ
    pub reason: Option<String>,
    /// ระบุว่าควรดำเนินการต่อหรือไม่ (ใช้ใน Stop Hook)
    pub continue_execution: Option<bool>,
    /// เหตุผลที่หยุด (ถ้า continue=false)
    pub stop_reason: Option<String>,
    /// ระงับการแสดงผลลัพธ์ของเครื่องมือหรือไม่
    pub suppress_output: Option<bool>,
    /// ข้อความระบบที่จะแสดงต่อผู้ใช้
    pub system_message: Option<String>,
    /// ข้อมูลเฉพาะของ Hook แต่ละประเภท (เช่น additionalContext, decision object)
    pub hook_specific_output: Option<HashMap<String, serde_json::Value>>,
}

impl HookOutput {
    /// สร้าง HookOutput เปล่า
    pub fn new() -> Self {
        Self::default()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ผลลัพธ์จากการรัน Hook แต่ละตัว (HookExecutionResult)
// ─────────────────────────────────────────────────────────────────────────────

/// ผลลัพธ์จากการเรียกใช้ Hook หนึ่งตัว
#[derive(Debug, Clone)]
pub struct HookExecutionResult {
    /// การทำงานสำเร็จหรือไม่ (ไม่มีข้อผิดพลาด)
    pub success: bool,
    /// ผลลัพธ์ที่ได้จาก Hook (ถ้ามี)
    pub output: Option<HookOutput>,
    /// ข้อผิดพลาดที่เกิดขึ้น (ถ้ามี)
    pub error: Option<anyhow::Error>,
    /// ระยะเวลาที่ใช้ในการทำงาน
    pub duration: Duration,
}

// ─────────────────────────────────────────────────────────────────────────────
// ผลลัพธ์รวมจาก Hook หลายตัว (AggregatedHookResult)
// ─────────────────────────────────────────────────────────────────────────────

/// ผลลัพธ์ที่ได้หลังจากรวม Hook หลายตัวเข้าด้วยกัน
#[derive(Debug, Clone)]
pub struct AggregatedHookResult {
    /// การทำงานของทุก Hook สำเร็จทั้งหมดหรือไม่
    pub success: bool,
    /// ผลลัพธ์ดิบจากทุก Hook
    pub all_outputs: Vec<HookOutput>,
    /// ข้อผิดพลาดที่เกิดขึ้นทั้งหมด
    pub errors: Vec<anyhow::Error>,
    /// เวลารวมที่ใช้ในการรันทุก Hook
    pub total_duration: Duration,
    /// ผลลัพธ์สุดท้ายหลังจากรวมตามกฎของเหตุการณ์นั้น ๆ
    pub final_output: Option<HookOutput>,
}

// ─────────────────────────────────────────────────────────────────────────────
// HookAggregator
// ─────────────────────────────────────────────────────────────────────────────

/// ตัวรวบรวมผลลัพธ์จาก Hook หลายตัว
/// ใช้กฎการรวมที่แตกต่างกันตามประเภทของเหตุการณ์
pub struct HookAggregator;

impl HookAggregator {
    /// รวมผลลัพธ์จากหลาย HookExecutionResult เข้าด้วยกัน
    ///
    /// # Arguments
    /// * `results` - รายการผลลัพธ์จากการเรียก Hook แต่ละตัว
    /// * `event_name` - ประเภทของเหตุการณ์ที่เกิดขึ้น
    ///
    /// # Returns
    /// AggregatedHookResult ที่มีผลลัพธ์รวมและข้อมูลสรุป
    pub fn aggregate_results(
        results: Vec<HookExecutionResult>,
        event_name: HookEventName,
    ) -> AggregatedHookResult {
        let mut all_outputs = Vec::new();
        let mut errors = Vec::new();
        let mut total_duration = Duration::ZERO;

        for result in results {
            total_duration += result.duration;

            if !result.success {
                if let Some(err) = result.error {
                    errors.push(err);
                }
            }

            if let Some(output) = result.output {
                all_outputs.push(output);
            }
        }

        let success = errors.is_empty();
        let final_output = Self::merge_outputs(&all_outputs, event_name);

        AggregatedHookResult {
            success,
            all_outputs,
            errors,
            total_duration,
            final_output,
        }
    }

    /// รวมผลลัพธ์หลายตัวตามประเภทของเหตุการณ์
    fn merge_outputs(
        outputs: &[HookOutput],
        event_name: HookEventName,
    ) -> Option<HookOutput> {
        if outputs.is_empty() {
            return None;
        }

        if outputs.len() == 1 {
            // ถ้ามีผลลัพธ์เดียว ส่งคืนโดยไม่ต้องรวม (แต่ยังต้องแปลงเป็นประเภทเฉพาะถ้าจำเป็น)
            return Some(outputs[0].clone());
        }

        let merged = match event_name {
            HookEventName::PreToolUse
            | HookEventName::PostToolUse
            | HookEventName::PostToolUseFailure
            | HookEventName::Stop
            | HookEventName::SubagentStop
            | HookEventName::UserPromptSubmit => {
                Self::merge_with_or_logic(outputs)
            }
            HookEventName::PermissionRequest => {
                Self::merge_permission_request_outputs(outputs)
            }
        };

        Some(merged)
    }

    /// รวมผลลัพธ์ด้วยตรรกะ OR (block/deny ชนะ) และต่อข้อความ
    ///
    /// กฎ:
    /// - decision "block" หรือ "deny" → ผลลัพธ์เป็น "block"
    /// - continue = false → มีผลเหนือ true
    /// - reason และ additionalContext ถูกนำมาต่อกันด้วยขึ้นบรรทัดใหม่
    fn merge_with_or_logic(outputs: &[HookOutput]) -> HookOutput {
        let mut merged = HookOutput::new();
        let mut reasons = Vec::new();
        let mut additional_contexts = Vec::new();
        let mut has_block = false;
        let mut has_continue_false = false;
        let mut stop_reason: Option<String> = None;
        let mut other_hook_specific_fields: HashMap<String, serde_json::Value> = HashMap::new();

        for output in outputs {
            // ตรวจสอบการตัดสินใจแบบ block/deny
            if let Some(dec) = &output.decision {
                if dec == "block" || dec == "deny" {
                    has_block = true;
                }
            }

            // เก็บเหตุผล
            if let Some(r) = &output.reason {
                reasons.push(r.clone());
            }

            // ตรวจสอบ continue flag
            if let Some(false) = output.continue_execution {
                has_continue_false = true;
                if let Some(sr) = &output.stop_reason {
                    stop_reason = Some(sr.clone());
                }
            }

            // ดึง additional context
            Self::extract_additional_context(output, &mut additional_contexts);

            // เก็บ hookSpecificOutput อื่น ๆ (ค่าหลังชนะ)
            if let Some(specific) = &output.hook_specific_output {
                for (key, value) in specific {
                    if key != "additionalContext" {
                        other_hook_specific_fields.insert(key.clone(), value.clone());
                    }
                }
            }

            // คัดลอกฟิลด์ธรรมดา (ค่าหลังชนะ)
            if output.suppress_output.is_some() {
                merged.suppress_output = output.suppress_output;
            }
            if output.system_message.is_some() {
                merged.system_message = output.system_message.clone();
            }
        }

        // ตั้งค่า decision สุดท้าย
        if has_block {
            merged.decision = Some("block".to_string());
        } else if outputs.iter().any(|o| o.decision.as_deref() == Some("allow")) {
            merged.decision = Some("allow".to_string());
        }

        // รวมเหตุผล
        if !reasons.is_empty() {
            merged.reason = Some(reasons.join("\n"));
        }

        // ตั้งค่า continue flag
        if has_continue_false {
            merged.continue_execution = Some(false);
            merged.stop_reason = stop_reason;
        }

        // สร้าง hook_specific_output
        let mut hook_specific = other_hook_specific_fields;
        if !additional_contexts.is_empty() {
            hook_specific.insert(
                "additionalContext".to_string(),
                serde_json::Value::String(additional_contexts.join("\n")),
            );
        }
        if !hook_specific.is_empty() {
            merged.hook_specific_output = Some(hook_specific);
        }

        merged
    }

    /// รวมผลลัพธ์สำหรับเหตุการณ์ PermissionRequest
    ///
    /// กฎ:
    /// - behavior: "deny" ชนะ "allow"
    /// - message: ต่อด้วยขึ้นบรรทัดใหม่
    /// - updatedInput: ใช้ค่าล่าสุด
    /// - updatedPermissions: รวมเป็นอาร์เรย์เดียว
    /// - interrupt: true ชนะ false
    fn merge_permission_request_outputs(outputs: &[HookOutput]) -> HookOutput {
        let mut merged = HookOutput::new();
        let mut messages = Vec::new();
        let mut has_deny = false;
        let mut has_allow = false;
        let mut interrupt = false;
        let mut updated_input: Option<serde_json::Value> = None;
        let mut all_updated_permissions: Vec<serde_json::Value> = Vec::new();

        for output in outputs {
            if let Some(specific) = &output.hook_specific_output {
                if let Some(decision_val) = specific.get("decision") {
                    if let Some(decision_obj) = decision_val.as_object() {
                        // ตรวจสอบ behavior
                        if let Some(behavior) = decision_obj.get("behavior").and_then(|v| v.as_str()) {
                            if behavior == "deny" {
                                has_deny = true;
                            } else if behavior == "allow" {
                                has_allow = true;
                            }
                        }

                        // เก็บ message
                        if let Some(msg) = decision_obj.get("message").and_then(|v| v.as_str()) {
                            messages.push(msg.to_string());
                        }

                        // ตรวจสอบ interrupt
                        if let Some(true) = decision_obj.get("interrupt").and_then(|v| v.as_bool()) {
                            interrupt = true;
                        }

                        // เก็บ updatedInput (ใช้ค่าล่าสุด)
                        if let Some(ui) = decision_obj.get("updatedInput") {
                            updated_input = Some(ui.clone());
                        }

                        // เก็บ updatedPermissions (รวมทั้งหมด)
                        if let Some(perms) = decision_obj.get("updatedPermissions").and_then(|v| v.as_array()) {
                            all_updated_permissions.extend(perms.clone());
                        }
                    }
                }
            }

            // คัดลอก continue และ reason
            if output.continue_execution.is_some() {
                merged.continue_execution = output.continue_execution;
            }
            if output.reason.is_some() {
                merged.reason = output.reason.clone();
            }
        }

        // สร้าง decision object สุดท้าย
        let mut merged_decision = serde_json::Map::new();

        if has_deny {
            merged_decision.insert("behavior".to_string(), serde_json::Value::String("deny".to_string()));
        } else if has_allow {
            merged_decision.insert("behavior".to_string(), serde_json::Value::String("allow".to_string()));
        }

        if !messages.is_empty() {
            merged_decision.insert("message".to_string(), serde_json::Value::String(messages.join("\n")));
        }

        if interrupt {
            merged_decision.insert("interrupt".to_string(), serde_json::Value::Bool(true));
        }

        if let Some(ui) = updated_input {
            merged_decision.insert("updatedInput".to_string(), ui);
        }

        if !all_updated_permissions.is_empty() {
            merged_decision.insert("updatedPermissions".to_string(), serde_json::Value::Array(all_updated_permissions));
        }

        // ใส่ decision object ลงใน hook_specific_output
        let mut hook_specific = merged.hook_specific_output.unwrap_or_default();
        hook_specific.insert("decision".to_string(), serde_json::Value::Object(merged_decision));
        merged.hook_specific_output = Some(hook_specific);

        merged
    }

    /// การรวมแบบง่ายสำหรับเหตุการณ์ที่ไม่มีตรรกะพิเศษ
    /// ใช้ค่าจากผลลัพธ์สุดท้าย แต่ต่อ additionalContext
    fn merge_simple(outputs: &[HookOutput]) -> HookOutput {
        let mut additional_contexts = Vec::new();
        let mut merged = HookOutput::new();

        for output in outputs {
            Self::extract_additional_context(output, &mut additional_contexts);
            // คัดลอกฟิลด์ทั้งหมดจาก output (ค่าหลังจะเขียนทับค่าก่อน)
            if let Some(dec) = &output.decision {
                merged.decision = Some(dec.clone());
            }
            if let Some(r) = &output.reason {
                merged.reason = Some(r.clone());
            }
            if let Some(cont) = output.continue_execution {
                merged.continue_execution = Some(cont);
            }
            if let Some(sr) = &output.stop_reason {
                merged.stop_reason = Some(sr.clone());
            }
            if let Some(so) = output.suppress_output {
                merged.suppress_output = Some(so);
            }
            if let Some(sm) = &output.system_message {
                merged.system_message = Some(sm.clone());
            }
            if let Some(specific) = &output.hook_specific_output {
                let mut current = merged.hook_specific_output.unwrap_or_default();
                for (k, v) in specific {
                    current.insert(k.clone(), v.clone());
                }
                merged.hook_specific_output = Some(current);
            }
        }

        // ต่อ additionalContext
        if !additional_contexts.is_empty() {
            let mut specific = merged.hook_specific_output.unwrap_or_default();
            specific.insert(
                "additionalContext".to_string(),
                serde_json::Value::String(additional_contexts.join("\n")),
            );
            merged.hook_specific_output = Some(specific);
        }

        merged
    }

    /// ดึง additionalContext จาก hook_specific_output (ถ้ามี)
    fn extract_additional_context(output: &HookOutput, contexts: &mut Vec<String>) {
        if let Some(specific) = &output.hook_specific_output {
            if let Some(val) = specific.get("additionalContext") {
                if let Some(s) = val.as_str() {
                    contexts.push(s.to_string());
                }
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn create_output(decision: Option<&str>, reason: Option<&str>) -> HookOutput {
        HookOutput {
            decision: decision.map(|s| s.to_string()),
            reason: reason.map(|s| s.to_string()),
            ..Default::default()
        }
    }

    #[test]
    fn test_merge_or_logic_block_wins() {
        let out1 = create_output(Some("allow"), Some("ok"));
        let out2 = create_output(Some("block"), Some("danger"));
        let merged = HookAggregator::merge_with_or_logic(&[out1, out2]);
        assert_eq!(merged.decision, Some("block".to_string()));
        assert_eq!(merged.reason, Some("ok\ndanger".to_string()));
    }

    #[test]
    fn test_aggregate_single_result() {
        let result = HookExecutionResult {
            success: true,
            output: Some(create_output(Some("allow"), Some("test"))),
            error: None,
            duration: Duration::from_millis(10),
        };
        let agg = HookAggregator::aggregate_results(vec![result], HookEventName::PreToolUse);
        assert!(agg.success);
        assert_eq!(agg.all_outputs.len(), 1);
        assert_eq!(agg.final_output.unwrap().decision, Some("allow".to_string()));
    }
}