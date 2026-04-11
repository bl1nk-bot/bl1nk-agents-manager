# Implementation Plan: Unified Registry, Monitoring & Honesty Layer

## Phase 1: Foundation - Registry Schema & Keyword Mapping [checkpoint: pending]

### 1.0 Task: Integrate bl1nk-keyword-validator
- [x] Task: Clone และวิเคราะห์ bl1nk-keyword-validator `#commit: 437d159`
    - [x] โคลน `https://github.com/billlzzz26/bl1nk-keyword-validator` ลงใน `vendor/bl1nk-keyword-validator/`
    - [x] วิเคราะห์โครงสร้าง: core/, cli/, schema.rs, validator.rs, search.rs
    - [x] สรุป dependencies และ functionality ที่มีอยู่
    - [x] เขียน analysis summary
- [~] Task: Merge keyword validator เข้ากับโปรเจค
    - [ ] โคลน `https://github.com/billlzzz26/bl1nk-keyword-validator` ลงใน `vendor/bl1nk-keyword-validator/`
    - [ ] วิเคราะห์โครงสร้าง: core/, cli/, schema.rs, validator.rs, search.rs
    - [ ] สรุป dependencies และ functionality ที่มีอยู่
    - [ ] เขียน analysis summary
- [ ] Task: Merge keyword validator เข้ากับโปรเจค
    - [ ] เพิ่ม `bl1nk-keyword-validator` เป็น workspace member หรือ vendored crate
    - [ ] ปรับ Cargo.toml ให้รองรับ dependency ร่วมกัน
    - [ ] ทดสอบว่า build ผ่านทั้งคู่
    - [ ] เขียน tests integration เบื้องต้น
- [ ] Task: Adapt validator สำหรับ registry ของเรา
    - [ ] เชื่อมต่อ validator กับ registry schema ที่มีอยู่
    - [ ] ปรับ CLI commands ให้เข้ากับ bl1nk-agents-manager
    - [ ] ใช้ validation logic เดิมแทนที่จะเขียนใหม่
    - [ ] เขียน tests validation ทำงานกับ registry ของเรา

### 1.1 Task: Define Unified Registry Schema
- [ ] Task: Create registry schema types
    - [ ] สร้าง `src/registry/mod.rs` พร้อม module structure
    - [ ] สร้าง types สำหรับ Registry, KeywordEntry, MonitoringRecord
    - [ ] เพิ่ม serde derive + JSON schema generation
    - [ ] เขียน unit tests สำหรับ schema types
- [ ] Task: Create JSON schema file
    - [ ] สร้าง `.config/registry-schema.json` พร้อม full schema definition
    - [ ] รวม keyword, monitoring, evidence, weight structures
    - [ ] เขียน test validate schema ถูกต้อง
- [ ] Task: Add schema validation
    - [ ] สร้าง `RegistryValidator` struct
    - [ ] implement validate_entry(), validate_registry() methods
    - [ ] เขียน tests สำหรับ validation rules

### 1.2 Task: Keyword Mapping & Basic Search
- [ ] Task: Implement keyword loading
    - [ ] สร้าง `RegistryService::load_from_file()` 
    - [ ] รองรับโหลดจาก `.config/registry.json` หรือ embedded default
    - [ ] เขียน tests โหลดไฟล์สำเร็จ/ไฟล์หาย
- [ ] Task: Implement basic keyword search
    - [ ] สร้าง `RegistryService::search_keywords()`
    - [ ] ค้นหา exact match + fuzzy match
    - [ ] return results พร้อม scores
    - [ ] เขียน tests ค้นหาได้ถูกต้อง
- [ ] Task: Create CLI search command
    - [ ] เพิ่ม `RegistryCommand::Search` ใน CLI
    - [ ] แสดงผลลัพธ์แบบ readable
    - [ ] เขียน tests CLI command ทำงาน

### 1.3 Task: Conductor - User Manual Verification 'Phase 1: Foundation' (Protocol in workflow.md)

---

## Phase 2: Semantic Search Prototype [checkpoint: pending]

### 2.1 Task: Implement Semantic Search (Keyword Overlap)
- [ ] Task: Create semantic search engine
    - [ ] สร้าง `SemanticSearchEngine` struct
    - [ ] Implement `find_semantic_matches()` ด้วย keyword overlap
    - [ ] คำนวณ semantic similarity จาก related terms, aliases
    - [ ] เขียน tests semantic search ได้ผลลัพธ์ถูกต้อง
- [ ] Task: Combine exact + semantic results
    - [ ] สร้าง `search_semantic()` ที่รวม exact + semantic
    - [ ] เรียงลำดับตาม combined score
    - [ ] เขียน tests ผลรวมถูกต้อง

### 2.2 Task: Integrate Semantic Search with CLI
- [ ] Task: Update CLI search command
    - [ ] เพิ่ม `--semantic` flag
    - [ ] แสดง semantic scores ในผลลัพธ์
    - [ ] เขียน tests CLI flag ทำงาน
- [ ] Task: Add search tests
    - [ ] เขียน integration tests สำหรับ search workflow
    - [ ] ครอบคลุม edge cases (empty query, no results)
    - [ ] Coverage >90%

### 2.3 Task: Conductor - User Manual Verification 'Phase 2: Semantic Search Prototype' (Protocol in workflow.md)

---

## Phase 3: Multi-Layer Monitoring [checkpoint: pending]

### 3.1 Task: Implement Monitoring Record Types
- [ ] Task: Create monitoring layer enums
    - [ ] สร้าง `MonitoringLayer` enum (Human, Model, Tool, Input, Output)
    - [ ] สร้าง `MonitoringRecord` struct
    - [ ] เพิ่ม serde + validation
    - [ ] เขียน tests สร้าง records ถูกต้อง
- [ ] Task: Implement evidence types
    - [ ] สร้าง `EvidenceRecord` enum (TestResults, FileChanges, CommandOutput, DecisionLog, ProgressMetrics, AlternativePath)
    - [ ] เขียน tests แต่ละ evidence type

### 3.2 Task: Implement Monitoring Recording
- [ ] Task: Create monitoring service
    - [ ] สร้าง `MonitoringService` struct
    - [ ] Implement `record_monitoring()` 
    - [ ] บันทึก monitoring records ลง JSON file
    - [ ] เขียน tests บันทึกและอ่านข้อมูลถูกต้อง
- [ ] Task: Implement monitoring queries
    - [ ] สร้าง `get_monitoring_for_agent()`, `get_monitoring_for_task()`
    - [ ] filter by layer, date range, quality score
    - [ ] เขียน tests query ได้ถูกต้อง

### 3.3 Task: Integrate Monitoring with CLI
- [ ] Task: Add monitoring CLI commands
    - [ ] เพิ่ม `RegistryCommand::Monitoring`
    - [ ] เพิ่ม `--layer`, `--task-id`, `--from`, `--to` flags
    - [ ] แสดง monitoring records แบบ readable
    - [ ] เขียน tests CLI commands
- [ ] Task: Add monitoring integration tests
    - [ ] เขียน tests end-to-end monitoring workflow
    - [ ] Coverage >90%

### 3.4 Task: Conductor - User Manual Verification 'Phase 3: Multi-Layer Monitoring' (Protocol in workflow.md)

---

## Phase 4: Dynamic Weight Calculation [checkpoint: pending]

### 4.1 Task: Implement Weight Calculation
- [ ] Task: Create weight calculation types
    - [ ] สร้าง `CalculatedWeight` struct
    - [ ] สร้าง weight components (base, self_claim, actual, tool, error, human)
    - [ ] เขียน tests struct fields ถูกต้อง
- [ ] Task: Implement weight calculation logic
    - [ ] สร้าง `CalculatedWeight::calculate()` method
    - [ ] คำนวณจาก monitoring data ตาม formula ใน spec
    - [ ] เขียน tests คำนวณน้ำหนักถูกต้อง
    - [ ] ครอบคลุม edge cases (no data, all success, all failures)

### 4.2 Task: Integrate Weight Calculation with Registry
- [ ] Task: Add weight recalculation
    - [ ] สร้าง `RegistryService::recalculate_weights()`
    - [ ] คำนวณใหม่ทุก keywords ที่มี monitoring data
    - [ ] บันทึกน้ำหนักกลับลง registry
    - [ ] เขียน tests recalculation ทำงาน
- [ ] Task: Add weight CLI command
    - [ ] เพิ่ม `RegistryCommand::Weights`
    - [ ] เพิ่ม `--show-calculation` flag แสดงวิธีคำนวณ
    - [ ] แสดงน้ำหนักแต่ละ keywords
    - [ ] เขียน tests CLI command

### 4.3 Task: Conductor - User Manual Verification 'Phase 4: Dynamic Weight Calculation' (Protocol in workflow.md)

---

## Phase 5: Evidence & Effectiveness Reports [checkpoint: pending]

### 5.1 Task: Implement Evidence Recording
- [ ] Task: Create evidence recording service
    - [ ] สร้าง `EvidenceService` struct
    - [ ] Implement `record_evidence()` 
    - [ ] เก็บ evidence พร้อม timestamp และ task context
    - [ ] เขียน tests บันทึก evidence ถูกต้อง
- [ ] Task: Implement evidence queries
    - [ ] สร้าง `get_evidence_for_task()`, `get_verified_evidence()`
    - [ ] filter by evidence type, verified status
    - [ ] เขียน tests query ได้ถูกต้อง

### 5.2 Task: Implement Effectiveness Reports
- [ ] Task: Create effectiveness report generator
    - [ ] สร้าง `EffectivenessReport` struct
    - [ ] สร้าง `generate_effectiveness_report()` method
    - [ ] คำนวณ before/after metrics
    - [ ] หา alternative paths available
    - [ ] คำนวณ path success rates
    - [ ] เขียน tests report generation ถูกต้อง
- [ ] Task: Add CLI for effectiveness
    - [ ] เพิ่ม `RegistryCommand::Effectiveness`
    - [ ] เพิ่ม `--period` flag (7d, 30d, 90d)
    - [ ] แสดง report แบบ readable
    - [ ] เขียน tests CLI command

### 5.3 Task: Conductor - User Manual Verification 'Phase 5: Evidence & Effectiveness Reports' (Protocol in workflow.md)

---

## Phase 6: Honesty Checks [checkpoint: pending]

### 6.1 Task: Implement Honesty Check System
- [ ] Task: Create honesty check types
    - [ ] สร้าง `HonestyReport` struct
    - [ ] สร้าง `DishonestyType` enum (ClaimWithoutEvidence, FalseCompletion, HighRiskSkip, UnapprovedCriticalDecision)
    - [ ] เขียน tests types ถูกต้อง
- [ ] Task: Implement honesty checks
    - [ ] สร้าง `check_honesty()` function
    - [ ] ตรวจสอบ claim without evidence
    - [ ] ตรวจสอบ false completion (claim done แต่ tests fail)
    - [ ] ตรวจสอบ high-risk skips
    - [ ] ตรวจสอบ unapproved decisions
    - [ ] เขียน tests แต่ละ check case

### 6.2 Task: Integrate Honesty with Monitoring
- [ ] Task: Record honesty violations
    - [ ] เมื่อ check_honesty พบปัญหา → บันทึกลง monitoring
    - [ ] สร้าง `HonestyViolation` monitoring record
    - [ ] เขียน tests บันทึก violations ถูกต้อง
- [ ] Task: Add honesty CLI commands
    - [ ] เพิ่ม `RegistryCommand::Honesty`
    - [ ] เพิ่ม `--agent-id`, `--violations-only` flags
    - [ ] แสดง honesty reports
    - [ ] เขียน tests CLI commands
- [ ] Task: Add honesty integration tests
    - [ ] เขียน tests end-to-end honesty workflow
    - [ ] Coverage >90%

### 6.3 Task: Conductor - User Manual Verification 'Phase 6: Honesty Checks' (Protocol in workflow.md)

---

## Phase 7: System Integration [checkpoint: pending]

### 7.1 Task: Integrate with Agent Routing
- [ ] Task: Update AgentRouter to use registry
    - [ ] แก้ไข `AgentRouter::route_task()` ใช้ semantic search
    - [ ] ใช้ calculated weights เลือก agent
    - [ ] บันทึก monitoring record สำหรับ routing decision
    - [ ] เขียน tests routing ใช้ registry data
- [ ] Task: Add routing monitoring
    - [ ] บันทึก routing decisions พร้อม reasoning
    - [ ] บันทึก success/failure ของ routing choices
    - [ ] เขียน tests routing monitoring ทำงาน

### 7.2 Task: Integrate with Permission System
- [ ] Task: Update PermissionManager to use monitoring
    - [ ] แก้ไข `PermissionManager::check_permission()` ใช้ monitoring data
    - [ ] ดู tool success rate, error rate, human approval
    - [ ] บันทึก permission decisions
    - [ ] เขียน tests permission ใช้ monitoring data
- [ ] Task: Add permission monitoring
    - [ ] บันทึก permission decisions พร้อมเหตุผล
    - [ ] เขียน tests permission monitoring ทำงาน

### 7.3 Task: Full System Integration Tests
- [ ] Task: Write integration tests
    - [ ] เขียน tests end-to-end workflow (routing → execution → monitoring → weights)
    - [ ] ครอบคลุม success และ failure cases
    - [ ] Coverage >90%

### 7.4 Task: Conductor - User Manual Verification 'Phase 7: System Integration' (Protocol in workflow.md)

---

## Phase 8: Documentation & Cleanup [checkpoint: pending]

### 8.1 Task: Update Documentation
- [ ] Task: Update README
    - [ ] เพิ่มส่วนอธิบาย Registry & Monitoring System
    - [ ] เพิ่ม usage examples
    - [ ] เพิ่ม CLI commands documentation
- [ ] Task: Create API docs
    - [ ] เพิ่ม rustdoc สำหรับ public API
    - [ ] เพิ่ม examples ใน docstrings

### 8.2 Task: Final Cleanup
- [ ] Task: Run full test suite
    - [ ] `cargo test --all-features` ผ่านทั้งหมด
    - [ ] `cargo clippy -- -D warnings` ไม่มี warnings
    - [ ] `cargo fmt --check` ผ่าน
    - [ ] Coverage >90%
- [ ] Task: Update conductor files
    - [ ] อัปเดต `todo.md` 
    - [ ] อัปเดต `plan.md` statuses

### 8.3 Task: Conductor - User Manual Verification 'Phase 8: Documentation & Cleanup' (Protocol in workflow.md)
