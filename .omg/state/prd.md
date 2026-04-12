# Product Requirements Document: bl1nk-agents Codebase Stabilization

## 1. Objective
สร้างความเสถียร (Stability) และความชัดเจน (Clarity) ให้กับฐานโค้ด Rust ของ `bl1nk-agents-manager` โดยการลบโค้ดที่ไม่ได้ใช้งานทั้งหมด (Dead Code) และเพิ่ม Unit Tests สำหรับส่วนงานที่สำคัญที่สุด

## 2. Acceptance Criteria
- [ ] **Zero Warnings**: `cargo check` ต้องไม่แสดงผลคำเตือนใดๆ (0 Warnings) รวมถึง dead_code, unused, และ unreachable patterns
- [ ] **Registry & Validator Integrity**: Unit Tests สำหรับ `RegistryService` และ `Validator` ครอบคลุมการทำงานหลัก (Search, Validate, Load) และผ่าน 100%
- [ ] **Orchestrator Stability**: Orchestrator Logic มีการทดสอบที่ยืนยันการคำนวณและลำดับความสำคัญของ Agent อย่างถูกต้อง
- [ ] **Clean Codebase**: ไม่มีการใช้ `#[allow(dead_code)]` เพื่อซ่อนคำเตือน และลบไฟล์หรือฟังก์ชันที่ไม่ได้เรียกใช้ออกทั้งหมด
- [ ] **Regression Free**: โค้ดที่เหลืออยู่ยังคงทำงานได้ตามปกติ (ทดสอบผ่าน Test Suite เดิมที่มีอยู่)

## 3. Non-goals
- การเพิ่มฟีเจอร์ใหม่ตามแผนงานใน `conductor/tracks/`
- การรีแฟคเตอร์ (Refactor) สถาปัตยกรรมของระบบใหม่ทั้งหมด (เน้นแค่คลีนอัปและทดสอบ)
- การเขียน Integration Tests ข้ามระบบในเฟสนี้ (เน้น Unit Tests เป็นหลัก)

## 4. Constraints
- **Environment**: ต้องทำงานบน Android/Termux (Rust/Cargo environment)
- **Strictness**: กฎ "Zero Warnings" ต้องถือเป็นเกณฑ์การยอมรับสูงสุด (Blocking Gate)
- **Sandbox**: ไม่มีการใช้ Docker ในขั้นตอนนี้ (ใช้ `--sandbox=none`)

## 5. Risk Factors
- **Aggressive Cleanup**: การลบโค้ดที่ไม่ได้ใช้อาจรวมถึงโครงสร้างที่เตรียมไว้เพื่ออนาคต (ยอมรับได้ตามความต้องการของผู้ใช้)
- **Integration Impact**: การคลีนอัปส่วน Orchestrator อาจกระทบกับการเรียกใช้งานในส่วนอื่นๆ หากมีจุดที่เรียกใช้ผ่าน Reflection หรือ Metadata (ต้องระวังเป็นพิเศษ)

## 6. Verification Method
- รันคำสั่ง `cargo check` และตรวจสอบว่าไม่มี Output (0 warnings)
- รันคำสั่ง `cargo test` และตรวจสอบว่าทุก Test Case ผ่าน (Success 100%)
- ตรวจสอบ `git diff` เพื่อยืนยันว่าไม่มีโค้ดที่ซ่อน Warnings ไว้ด้วย Annotation

---
**Status**: LOCKED
**Version**: 1.0.0
