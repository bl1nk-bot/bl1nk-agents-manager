# Specification: Unified Registry, Monitoring & Honesty Layer

## Overview

สร้างระบบ **Registry & Monitoring Layer** ที่ทำหน้าที่:

1. **Keyword → Meaning → Weight Mapping** - เชื่อมโยงคำค้นกับความหมาย + semantic search + คำนวณค่าน้ำหนัก
2. **System-Wide Monitoring** - ตรวจจับจาก people, models, tools, inputs, outputs (ไม่ใช่แค่โมเดลอย่างเดียว)
3. **Evidence & Proof System** - พิสูจน์ว่าระบบทำให้โมเดลดีขึ้นจริง ไม่ใช่ต้อนไปทางตัน
4. **Dynamic Weight Calculation** - คำนวณค่าน้ำหนักจากพฤติกรรมจริง ไม่ใช่แค่กำหนดตายตัว

## Core Principles

### ระบบต้องพิสูจน์ได้ว่า

- ✅ **ทำให้โมเดลดีขึ้นจริง** - มี metrics เปรียบเทียบก่อน-หลัง
- ✅ **ไม่ต้อนโมเดลไปทางตัน** - มี alternative paths และ success rates
- ✅ **โปร่งใสทุกชั้น** - people, model, tool, input, output ถูกล็อกหมด
- ✅ **ค่าน้ำหนักมีเหตุผล** - คำนวณจากพฤติกรรมจริง ไม่ใช่ hardcoded

### ระบบต้องบันทึก

- 📊 **โมเดลประกาศตัวเอง** - พูดว่าถูก vs ถูกจริง ต่างกันแค่ไหน
- 🔧 **Tool Usage** - เรียก tool ไหน บ่อยแค่ไหน เพราะอะไร เกิด error อย่างไร
- 👤 **Human Interactions** - ผู้ใช้ตัดสินใจอย่างไร ตอบสนองอย่างไร
- 📈 **Trends Over Time** - ดีขึ้นหรือแย่ลง

## Goals

1. **Semantic Understanding:** เข้าใจความหมายตรงกันผ่าน semantic search + keyword mapping
2. **Multi-Layer Monitoring:** ตรวจจับทุกชั้น (people, model, tool, input, output)
3. **Evidence of Effectiveness:** พิสูจน์ว่าระบบทำให้โมเดลดีขึ้น
4. **Dynamic Weight Calculation:** คำนวณ weights จากพฤติกรรมจริง

## Functional Requirements

### 1. Keyword & Semantic Registry

#### 1.1 Structure with Semantic Search

```json
{
  "registry": {
    "version": "1.0.0",
    "keywords": [
      {
        "id": "delegate-task",
        "term": "delegate",
        "aliases": ["assign", "route", "send to agent"],
        "semanticEmbeddings": {
          "vector": [0.12, -0.34, 0.56, ...],
          "model": "text-embedding-3-small"
        },
        "meanings": [
          {
            "context": "routing",
            "definition": "มอบหมายงานให้ agent อื่นดำเนินการ",
            "relatedTerms": ["agent", "routing", "task"]
          },
          {
            "context": "management",
            "definition": "จัดการงานโดยมอบให้ผู้อื่น",
            "relatedTerms": ["manage", "coordinate"]
          }
        ],
        "expectedEvidence": ["agent_id selected", "routing decision log"],
        "baseWeight": 0.8,
        "calculatedWeight": 0.0  // จะคำนวณจากพฤติกรรม
      }
    ]
  }
}
```

#### 1.2 Semantic Search

```rust
pub struct SemanticSearchEngine {
    // สำหรับ prototype: ใช้ keyword-based + cosine similarity
    // สำหรับ production: ใช้ embeddings จาก API
    index: HashMap<String, KeywordEntry>,
}

impl SemanticSearchEngine {
    /// ค้นหาด้วยความหมาย ไม่ใช่แค่คำตรงกัน
    pub fn search_semantic(&self, query: &str) -> Vec<SemanticResult> {
        // 1. หา keywords ที่ตรงกันแบบ exact/fuzzy
        let exact_matches = self.find_exact_matches(query);
        
        // 2. หา keywords ที่เกี่ยวข้องผ่าน semantic similarity
        let semantic_matches = self.find_semantic_matches(query);
        
        // 3. รวมและเรียงตามคะแนน
        let mut results = vec![];
        results.extend(exact_matches);
        results.extend(semantic_matches);
        results.sort_by(|a, b| b.combined_score.partial_cmp(&a.combined_score).unwrap());
        results
    }
    
    /// คำนวณ semantic similarity (prototype: ใช้ keyword overlap)
    fn find_semantic_matches(&self, query: &str) -> Vec<SemanticResult> {
        self.index.values()
            .filter(|entry| {
                // ตรวจสอบ related terms, aliases, meanings
                entry.meanings.iter().any(|m| 
                    m.related_terms.iter().any(|t| 
                        query.contains(t) || t.contains(query)
                    )
                )
            })
            .map(|entry| SemanticResult {
                entry: entry.clone(),
                semantic_score: self.calculate_similarity(query, entry),
                match_type: MatchType::Semantic,
            })
            .collect()
    }
}
```

### 2. Multi-Layer Monitoring

#### 2.1 Monitoring Layers

```rust
pub enum MonitoringLayer {
    /// ผู้ใช้โต้ตอบอย่างไร ตอบสนองต่อข้อเสนอแนะแค่ไหน
    Human {
        user_id: String,
        action: HumanAction,  // approve, reject, modify, ignore
        response_time: Duration,
        modification_details: Option<String>,
    },
    
    /// โมเดลทำอะไร ประกาศตัวเองอย่างไร ผลจริงเป็นอย่างไร
    Model {
        agent_id: String,
        claim: String,              // สิ่งที่โมเดลอ้าง
        actual_result: String,      // ผลจริง
        self_assessment: String,    // โมเดลประเมินตัวเอง
        objective_assessment: String, // ประเมินจริงจาก outside
        confidence: f32,
    },
    
    /// Tool ถูกเรียกอย่างไร สำเร็จหรือล้มเหลว เพราะอะไร
    Tool {
        tool_name: String,
        input: String,
        output: String,
        success: bool,
        error_type: Option<String>,
        execution_time: Duration,
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
```

#### 2.2 Monitoring Records

```rust
pub struct MonitoringRecord {
    pub id: String,
    pub timestamp: DateTime,
    pub layer: MonitoringLayer,
    pub task_id: Option<String>,
    pub session_id: String,
    
    // สิ่งที่คาดว่าจะเกิด
    pub expected: Option<String>,
    // สิ่งที่เกิดขึ้นจริง
    pub actual: String,
    // ความแตกต่าง
    pub delta: Option<String>,
    
    // คะแนนคุณภาพ
    pub quality_score: f32,  // 0.0-1.0
    
    // evidences
    pub evidence: Vec<EvidenceRecord>,
}
```

### 3. Evidence & Proof System

#### 3.1 Evidence Types

```rust
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
        execution_time: Duration,
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
        trend: Trend,  // Improving, Stable, Degrading
    },
    
    /// Alternative Path - แสดงว่ามีทางอื่น ไม่ได้ต้อนไปทางตัน
    AlternativePath {
        original_path: String,
        alternative_path: String,
        why_alternative: String,
        success_rate_comparison: Option<(f32, f32)>,
    },
}

pub enum Trend {
    Improving,
    Stable,
    Degrading,
}
```

#### 3.2 Proof of Effectiveness

```rust
pub struct EffectivenessReport {
    pub period: (DateTime, DateTime),
    
    // Metrics ก่อนและหลังใช้ระบบ
    pub metrics: Vec<MetricComparison>,
    
    // แสดงว่ามีทางเลือก ไม่ใช่ทางตัน
    pub alternative_paths_available: usize,
    pub path_success_rates: Vec<(String, f32)>,
    
    // สรุป
    pub overall_assessment: Assessment,
    pub confidence_level: f32,
}

pub struct MetricComparison {
    pub name: String,
    pub before: f32,
    pub after: f32,
    pub change_percentage: f32,
    pub trend: Trend,
    pub evidence: Vec<EvidenceRecord>,
}

pub enum Assessment {
    SignificantlyImproved,
    ModeratelyImproved,
    NoChange,
    Degraded,
}
```

### 4. Dynamic Weight Calculation

#### 4.1 Weight Components

```rust
pub struct CalculatedWeight {
    pub base_weight: f32,           // น้ำหนักพื้นฐานจาก schema
    pub self_claim_weight: f32,     // จากที่โมเดลประกาศตัวเอง
    pub actual_performance: f32,    // จากผลจริง
    pub tool_success_rate: f32,     // อัตราสำเร็จของ tool
    pub error_frequency: f32,       // ความถี่ error
    pub human_approval_rate: f32,   // อัตราอนุมัติจากผู้ใช้
    
    // น้ำหนักสุดท้าย
    pub final_weight: f32,
}

impl CalculatedWeight {
    pub fn calculate(entry: &KeywordEntry, monitoring_data: &[MonitoringRecord]) -> Self {
        // 1. base weight จาก schema
        let base_weight = entry.base_weight;
        
        // 2. self-claim weight: โมเดลบอกว่าตัวเองถูกแค่ไหน
        let model_claims = monitoring_data.iter()
            .filter(|r| matches!(r.layer, MonitoringLayer::Model { .. }))
            .filter(|r| {
                if let MonitoringLayer::Model { self_assessment, .. } = &r.layer {
                    self_assessment.contains("success") || self_assessment.contains("correct")
                } else { false }
            })
            .count() as f32;
        
        let self_claim_weight = if model_claims > 0.0 {
            model_claims / monitoring_data.len() as f32
        } else {
            base_weight
        };
        
        // 3. actual performance: ผลจริงเป็นอย่างไร
        let actual_success = monitoring_data.iter()
            .filter(|r| r.quality_score > 0.7)
            .count() as f32 / monitoring_data.len() as f32;
        
        // 4. tool success rate
        let tool_calls = monitoring_data.iter()
            .filter(|r| matches!(r.layer, MonitoringLayer::Tool { .. }));
        
        let tool_success_rate = tool_calls.clone()
            .filter(|r| {
                if let MonitoringLayer::Tool { success, .. } = &r.layer {
                    *success
                } else { false }
            })
            .count() as f32 / tool_calls.count() as f32.max(1.0);
        
        // 5. error frequency (ยิ่งบ่อย ยิ่งลดน้ำหนัก)
        let error_rate = monitoring_data.iter()
            .filter(|r| r.quality_score < 0.5)
            .count() as f32 / monitoring_data.len() as f32;
        
        // 6. human approval rate
        let human_approvals = monitoring_data.iter()
            .filter(|r| matches!(r.layer, MonitoringLayer::Human { action: HumanAction::Approve, .. }));
        
        let human_approval_rate = human_approvals.count() as f32 
            / monitoring_data.iter()
                .filter(|r| matches!(r.layer, MonitoringLayer::Human { .. }))
                .count() as f32
            .max(1.0);
        
        // คำนวณน้ำหนักสุดท้าย (weighted average)
        let final_weight = (
            base_weight * 0.1
            + self_claim_weight * 0.15
            + actual_success * 0.30
            + tool_success_rate * 0.20
            + (1.0 - error_rate) * 0.15
            + human_approval_rate * 0.10
        ).clamp(0.0, 1.0);
        
        Self {
            base_weight,
            self_claim_weight,
            actual_performance: actual_success,
            tool_success_rate,
            error_frequency: error_rate,
            human_approval_rate,
            final_weight,
        }
    }
}
```

#### 4.2 Weight Recalculation Trigger

- คำนวณใหม่ทุกครั้งที่มีการบันทึก monitoring record ใหม่
- หรือคำนวณ batch ทุก 100 records
- เก็บประวัติการเปลี่ยนแปลงน้ำหนักเพื่อดู trend

### 5. Integration Points

#### 5.1 With Routing

```rust
// ใน AgentRouter
let keyword_results = registry.search_semantic(&user_prompt);

// คำนวณน้ำหนักใหม่สำหรับ keywords ที่ใช้
for result in &keyword_results {
    let monitoring_data = registry.get_monitoring_for_keyword(&result.entry.id);
    let calculated_weight = CalculatedWeight::calculate(&result.entry, &monitoring_data);
    
    // ใช้น้ำหนักที่คำนวณได้เลือก agent
    if calculated_weight.final_weight > 0.7 {
        // keyword นี้เชื่อถือได้ เลือก agent ตาม mapping
    } else if calculated_weight.final_weight < 0.3 {
        // keyword นี้มีปัญหา ต้องหาทางเลือกอื่น
        let alternatives = registry.find_alternative_keywords(&result.entry.id);
        // ... พยายามใช้ keywords อื่น
    }
}
```

#### 5.2 With Task Execution

```rust
// เมื่อ task เสร็จ - ต้องเก็บ monitoring records
let monitoring_records = vec![
    MonitoringRecord {
        layer: MonitoringLayer::Model {
            agent_id: agent_id.clone(),
            claim: "Task completed".into(),
            actual_result: test_summary,
            self_assessment: "All tests passing".into(),
            objective_assessment: objective_test_result,
            confidence: 0.9,
        },
        // ...
    },
    MonitoringRecord {
        layer: MonitoringLayer::Tool {
            tool_name: "cargo test".into(),
            input: "--all-features".into(),
            output: test_output,
            success: all_passed,
            // ...
        },
        // ...
    },
    MonitoringRecord {
        layer: MonitoringLayer::Human {
            user_id: user_id.clone(),
            action: HumanAction::Approve,
            // ...
        },
        // ...
    },
];

registry.record_monitoring(monitoring_records)?;

// คำนวณน้ำหนักใหม่
registry.recalculate_weights()?;

// สร้าง effectiveness report
let report = registry.generate_effectiveness_report(period)?;
```

#### 5.3 With Permission System

```rust
// Permission ดู monitoring history
pub fn check_permission(agent_id: &str, tool: &str) -> PermissionResult {
    let monitoring = registry.get_monitoring_for_agent(agent_id);
    
    // ดู tool success rate
    let tool_success_rate = calculate_tool_success_rate(&monitoring, tool);
    
    // ดู error frequency
    let error_rate = calculate_error_rate(&monitoring);
    
    // ดู human approval rate
    let approval_rate = calculate_human_approval_rate(&monitoring);
    
    // ตัดสินใจตาม thresholds
    if tool_success_rate < 0.5 {
        return PermissionResult::RequireApproval {
            reason: format!("Tool success rate only {:.1}%", tool_success_rate * 100.0),
        };
    }
    
    if error_rate > 0.3 {
        return PermissionResult::RequireApproval {
            reason: format!("High error rate: {:.1}%", error_rate * 100.0),
        };
    }
    
    PermissionResult::Allow
}
```

### 6. CLI Commands

```rust
#[derive(clap::Subcommand)]
pub enum RegistryCommand {
    /// เพิ่ม keyword mapping
    AddKeyword {
        term: String,
        #[arg(short, long)]
        aliases: Option<Vec<String>>,
        #[arg(long)]
        base_weight: Option<f32>,
    },
    
    /// Semantic search
    Search {
        query: String,
        #[arg(long)]
        semantic: bool,
        #[arg(long)]
        show_weights: bool,
    },
    
    /// แสดง monitoring records
    Monitoring {
        #[arg(short, long)]
        layer: Option<String>,  // human, model, tool, input, output
        #[arg(short, long)]
        task_id: Option<String>,
        #[arg(long)]
        from: Option<DateTime>,
        #[arg(long)]
        to: Option<DateTime>,
    },
    
    /// แสดง weights ที่คำนวณได้
    Weights {
        #[arg(short, long)]
        keyword: Option<String>,
        #[arg(long)]
        show_calculation: bool,
    },
    
    /// แสดง effectiveness report
    Effectiveness {
        #[arg(long)]
        period: Option<String>,  // "7d", "30d", "90d"
        #[arg(long)]
        show_alternatives: bool,
    },
    
    /// แสดง honesty checks
    Honesty {
        #[arg(short, long)]
        agent_id: Option<String>,
        #[arg(long)]
        violations_only: bool,
    },
    
    /// Recalculate weights
    RecalculateWeights,
    
    /// Export registry
    Export {
        #[arg(long, default_value = "json")]
        format: String,
    },
}
```

### 7. Non-Functional Requirements

- **Semantic Search:** <100ms สำหรับ 1000 keywords (prototype: keyword overlap)
- **Monitoring Recording:** <10ms ต่อ record
- **Weight Calculation:** <50ms สำหรับ recalculation
- **Storage:** JSON file ไม่ควรเกิน 10MB (archive เก่าถ้าเกิน)
- **Report Generation:** <500ms สำหรับ effectiveness report

### 8. Acceptance Criteria

- [ ] Semantic search (keyword overlap prototype) ทำงานได้
- [ ] Monitoring records ทั้ง 5 layers (human, model, tool, input, output)
- [ ] Evidence recording ครบทุก type
- [ ] Weight calculation จาก monitoring data
- [ ] Effectiveness report แสดง before/after metrics
- [ ] Alternative paths tracking (ไม่ได้ต้อนไปทางตัน)
- [ ] Honesty checks ทำงานได้
- [ ] CLI commands ครบ
- [ ] Integration กับ routing, permission, task execution
- [ ] Tests >90% coverage

### 9. Out of Scope

- ❌ Full ML embeddings (ใช้ keyword overlap prototype ก่อน)
- ❌ Web UI
- ❌ Real-time distributed monitoring
- ❌ Automated decision making (ระบบแค่บันทึกและคำนวณ คนตัดสินใจ)
