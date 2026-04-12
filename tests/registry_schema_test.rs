//! Tests สำหรับ Unified Registry Schema - Phase 1, Task 1.1
//!
//! ทดสอบ schema types ตาม spec:
//! - Registry, KeywordEntry, MonitoringRecord
//! - EvidenceRecord และ types ที่เกี่ยวข้อง
//! - serde serialization/deserialization
//! - JSON schema generation

use bl1nk_agents_manager::registry::schema::{
    Registry, KeywordEntry, KeywordMeaning, MonitoringRecord, MonitoringLayer,
    EvidenceRecord, CalculatedWeight, Trend, Assessment, HumanAction,
};
use chrono::Utc;
use serde_json;

#[cfg(test)]
mod schema_type_tests {
    use super::*;

    #[test]
    fn test_keyword_entry_creation() {
        // ทดสอบสร้าง KeywordEntry
        let entry = KeywordEntry {
            id: "delegate-task".to_string(),
            term: "delegate".to_string(),
            aliases: vec!["assign".to_string(), "route".to_string()],
            meanings: vec![
                KeywordMeaning {
                    context: "routing".to_string(),
                    definition: "มอบหมายงานให้ agent อื่นดำเนินการ".to_string(),
                    related_terms: vec!["agent".to_string(), "routing".to_string()],
                }
            ],
            expected_evidence: vec!["agent_id selected".to_string()],
            base_weight: 0.8,
            calculated_weight: 0.0,
        };

        assert_eq!(entry.id, "delegate-task");
        assert_eq!(entry.term, "delegate");
        assert_eq!(entry.aliases.len(), 2);
        assert_eq!(entry.meanings.len(), 1);
        assert_eq!(entry.base_weight, 0.8);
        assert_eq!(entry.calculated_weight, 0.0);
    }

    #[test]
    fn test_keyword_entry_serialization() {
        // ทดสอบ serialize KeywordEntry เป็น JSON
        let entry = KeywordEntry {
            id: "test-keyword".to_string(),
            term: "test".to_string(),
            aliases: vec![],
            meanings: vec![],
            expected_evidence: vec![],
            base_weight: 0.5,
            calculated_weight: 0.0,
        };

        let json = serde_json::to_string(&entry).expect("Should serialize");
        assert!(json.contains("test-keyword"));
        assert!(json.contains("0.5"));

        // ทดสอบ deserialize กลับ
        let deserialized: KeywordEntry = serde_json::from_str(&json).expect("Should deserialize");
        assert_eq!(deserialized.id, "test-keyword");
        assert_eq!(deserialized.base_weight, 0.5);
    }

    #[test]
    fn test_registry_creation() {
        // ทดสอบสร้าง Registry
        let registry = Registry {
            version: "1.0.0".to_string(),
            keywords: vec![],
        };

        assert_eq!(registry.version, "1.0.0");
        assert!(registry.keywords.is_empty());
    }

    #[test]
    fn test_registry_with_keywords() {
        // ทดสอบ Registry ที่มี keywords
        let keyword = KeywordEntry {
            id: "search".to_string(),
            term: "search".to_string(),
            aliases: vec!["find".to_string()],
            meanings: vec![
                KeywordMeaning {
                    context: "general".to_string(),
                    definition: "ค้นหาข้อมูล".to_string(),
                    related_terms: vec!["find".to_string()],
                }
            ],
            expected_evidence: vec![],
            base_weight: 0.7,
            calculated_weight: 0.0,
        };

        let registry = Registry {
            version: "1.0.0".to_string(),
            keywords: vec![keyword.clone()],
        };

        assert_eq!(registry.keywords.len(), 1);
        assert_eq!(registry.keywords[0].id, "search");
    }

    #[test]
    fn test_registry_serialization() {
        // ทดสอบ serialize/deserialize Registry
        let registry = Registry {
            version: "1.0.0".to_string(),
            keywords: vec![],
        };

        let json = serde_json::to_string(&registry).expect("Should serialize");
        assert!(json.contains("1.0.0"));

        let deserialized: Registry = serde_json::from_str(&json).expect("Should deserialize");
        assert_eq!(deserialized.version, "1.0.0");
    }

    #[test]
    fn test_monitoring_layer_human() {
        // ทดสอบ MonitoringLayer::Human
        let layer = MonitoringLayer::Human {
            user_id: "user-1".to_string(),
            action: HumanAction::Approve,
            response_time: std::time::Duration::from_secs(5),
            modification_details: None,
        };

        match &layer {
            MonitoringLayer::Human { user_id, action, .. } => {
                assert_eq!(user_id, "user-1");
                assert_eq!(*action, HumanAction::Approve);
            }
            _ => panic!("Expected Human layer"),
        }
    }

    #[test]
    fn test_monitoring_layer_model() {
        // ทดสอบ MonitoringLayer::Model
        let layer = MonitoringLayer::Model {
            agent_id: "agent-1".to_string(),
            claim: "Task completed successfully".to_string(),
            actual_result: "Tests passing".to_string(),
            self_assessment: "All correct".to_string(),
            objective_assessment: "Verified".to_string(),
            confidence: 0.9,
        };

        match &layer {
            MonitoringLayer::Model { agent_id, confidence, .. } => {
                assert_eq!(agent_id, "agent-1");
                assert!((*confidence - 0.9).abs() < 0.001);
            }
            _ => panic!("Expected Model layer"),
        }
    }

    #[test]
    fn test_monitoring_layer_tool() {
        // ทดสอบ MonitoringLayer::Tool
        let layer = MonitoringLayer::Tool {
            tool_name: "cargo test".to_string(),
            input: "--all-features".to_string(),
            output: "All tests passed".to_string(),
            success: true,
            error_type: None,
            execution_time: std::time::Duration::from_millis(1500),
            retry_count: 0,
        };

        match &layer {
            MonitoringLayer::Tool { tool_name, success, .. } => {
                assert_eq!(tool_name, "cargo test");
                assert!(*success);
            }
            _ => panic!("Expected Tool layer"),
        }
    }

    #[test]
    fn test_monitoring_record_creation() {
        // ทดสอบสร้าง MonitoringRecord
        let record = MonitoringRecord {
            id: "rec-001".to_string(),
            timestamp: Utc::now().to_rfc3339(),
            layer: MonitoringLayer::Tool {
                tool_name: "test".to_string(),
                input: "".to_string(),
                output: "ok".to_string(),
                success: true,
                error_type: None,
                execution_time: std::time::Duration::from_secs(1),
                retry_count: 0,
            },
            task_id: Some("task-1".to_string()),
            session_id: "session-1".to_string(),
            expected: Some("Success".to_string()),
            actual: "Passed".to_string(),
            delta: None,
            quality_score: 0.95,
            evidence: vec![],
        };

        assert_eq!(record.id, "rec-001");
        assert_eq!(record.quality_score, 0.95);
        assert!(record.task_id.is_some());
    }

    #[test]
    fn test_evidence_record_test_results() {
        // ทดสอบ EvidenceRecord::TestResults
        let evidence = EvidenceRecord::TestResults {
            command: "cargo test".to_string(),
            output: "5 passed".to_string(),
            pass_count: 5,
            fail_count: 0,
            coverage_percentage: Some(92.5),
        };

        match &evidence {
            EvidenceRecord::TestResults { pass_count, fail_count, coverage_percentage, .. } => {
                assert_eq!(*pass_count, 5);
                assert_eq!(*fail_count, 0);
                assert_eq!(*coverage_percentage, Some(92.5));
            }
            _ => panic!("Expected TestResults"),
        }
    }

    #[test]
    fn test_evidence_record_file_changes() {
        // ทดสอบ EvidenceRecord::FileChanges
        let evidence = EvidenceRecord::FileChanges {
            files: vec!["src/registry/mod.rs".to_string()],
            diff_summary: "Added schema types".to_string(),
            lines_added: 100,
            lines_removed: 10,
        };

        match &evidence {
            EvidenceRecord::FileChanges { files, lines_added, .. } => {
                assert_eq!(files.len(), 1);
                assert_eq!(*lines_added, 100);
            }
            _ => panic!("Expected FileChanges"),
        }
    }

    #[test]
    fn test_evidence_record_decision_log() {
        // ทดสอบ EvidenceRecord::DecisionLog
        let evidence = EvidenceRecord::DecisionLog {
            decision: "Use BM25 for routing".to_string(),
            reasoning: "Better than keyword match".to_string(),
            alternatives_considered: vec!["Exact match".to_string()],
            why_this_choice: "More accurate".to_string(),
            risk_assessment: Some("Low risk".to_string()),
        };

        match &evidence {
            EvidenceRecord::DecisionLog { decision, alternatives_considered, .. } => {
                assert_eq!(decision, "Use BM25 for routing");
                assert_eq!(alternatives_considered.len(), 1);
            }
            _ => panic!("Expected DecisionLog"),
        }
    }

    #[test]
    fn test_calculated_weight() {
        // ทดสอบ CalculatedWeight
        let weight = CalculatedWeight {
            base_weight: 0.8,
            self_claim_weight: 0.9,
            actual_performance: 0.85,
            tool_success_rate: 0.95,
            error_frequency: 0.05,
            human_approval_rate: 0.9,
            final_weight: 0.88,
        };

        assert!((weight.base_weight - 0.8).abs() < 0.001);
        assert!((weight.final_weight - 0.88).abs() < 0.001);
        assert!(weight.final_weight >= 0.0 && weight.final_weight <= 1.0);
    }

    #[test]
    fn test_trend_enum() {
        // ทดสอบ Trend enum
        let improving = Trend::Improving;
        let stable = Trend::Stable;
        let degrading = Trend::Degrading;

        assert!(matches!(improving, Trend::Improving));
        assert!(matches!(stable, Trend::Stable));
        assert!(matches!(degrading, Trend::Degrading));
    }

    #[test]
    fn test_assessment_enum() {
        // ทดสอบ Assessment enum
        let assessments = vec![
            Assessment::SignificantlyImproved,
            Assessment::ModeratelyImproved,
            Assessment::NoChange,
            Assessment::Degraded,
        ];

        assert_eq!(assessments.len(), 4);
    }

    #[test]
    fn test_human_action_enum() {
        // ทดสอบ HumanAction enum
        let actions = vec![
            HumanAction::Approve,
            HumanAction::Reject,
            HumanAction::Modify,
            HumanAction::Ignore,
        ];

        assert_eq!(actions.len(), 4);
    }

    #[test]
    fn test_json_schema_generation() {
        // ทดสอบสร้าง JSON schema จาก Registry type
        let schema = schemars::schema_for!(Registry);
        
        // schemars 1.0 ใช้ to_value()
        let value = serde_json::to_value(&schema).expect("Should serialize schema");
        assert!(value.get("title").is_some());
        assert_eq!(value.get("title").unwrap().as_str().unwrap(), "Registry");
    }

    #[test]
    fn test_json_schema_keyword_entry() {
        // ทดสอบสร้าง JSON schema จาก KeywordEntry
        let schema = schemars::schema_for!(KeywordEntry);
        
        let value = serde_json::to_value(&schema).expect("Should serialize schema");
        assert_eq!(value.get("title").unwrap().as_str().unwrap(), "KeywordEntry");
    }

    #[test]
    fn test_json_schema_monitoring_record() {
        // ทดสอบสร้าง JSON schema จาก MonitoringRecord
        let schema = schemars::schema_for!(MonitoringRecord);
        
        let value = serde_json::to_value(&schema).expect("Should serialize schema");
        assert!(value.get("title").is_some());
    }

    #[test]
    fn test_json_schema_evidence_record() {
        // ทดสอบสร้าง JSON schema จาก EvidenceRecord
        let schema = schemars::schema_for!(EvidenceRecord);
        
        let value = serde_json::to_value(&schema).expect("Should serialize schema");
        assert!(value.get("title").is_some());
    }

    #[test]
    fn test_validate_schema_file_exists() {
        // ทดสอบว่า schema file ถูกสร้างแล้ว
        let schema_path = ".config/registry-schema.json";
        assert!(
            std::path::Path::new(schema_path).exists(),
            "Schema file should exist at {}",
            schema_path
        );
    }

    #[test]
    fn test_validate_schema_file_is_valid_json() {
        // ทดสอบว่า schema file เป็น JSON ที่ถูกต้อง
        let schema_path = ".config/registry-schema.json";
        let content = std::fs::read_to_string(schema_path).expect("Should read schema file");
        let _: serde_json::Value = serde_json::from_str(&content).expect("Should be valid JSON");
    }

    #[test]
    fn test_registry_validates_against_schema() {
        // ทดสอบว่า Registry validate กับ schema
        let schema_path = ".config/registry-schema.json";
        let schema_content = std::fs::read_to_string(schema_path).expect("Should read schema file");
        let schema: serde_json::Value = serde_json::from_str(&schema_content).expect("Should parse schema");
        
        let registry = Registry {
            version: "1.0.0".to_string(),
            keywords: vec![],
        };
        
        let registry_json = serde_json::to_value(&registry).expect("Should serialize registry");
        
        // ตรวจสอบว่า schema มี title = "Registry"
        assert_eq!(
            schema.get("title").and_then(|v| v.as_str()).unwrap(),
            "Registry"
        );
        
        // ตรวจสอบว่า registry JSON ตรงกับ schema structure
        assert!(registry_json.get("version").is_some());
        assert!(registry_json.get("keywords").is_some());
    }
}
