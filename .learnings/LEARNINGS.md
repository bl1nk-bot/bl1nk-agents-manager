# Learnings

Corrections, insights, and knowledge gaps captured during development.

**Categories**: correction | insight | knowledge_gap | best_practice
**Areas**: frontend | backend | infra | tests | docs | config
**Statuses**: pending | in_progress | resolved | wont_fix | promoted | promoted_to_skill

---

## [LRN-20260420-001] best_practice - CI/CD Modernization

**บันทึกเมื่อ**: 2026-04-20T02:00:00Z
**ลำดับความสำคัญ**: สูง
**สถานะ**: promoted
**ขอบเขต**: infra

### Summary
อัปเกรด GitHub Actions เพื่อรองรับ Node.js 24 และกำจัดคำเตือน Deprecation

### Details
- GitHub ประกาศเลิกใช้ Node.js 20 ใน runner เร็วๆ นี้
- `actions-rs/toolchain` ล้าสมัยและใช้คำสั่ง `set-output` ที่เลิกใช้แล้ว
- ต้องเปลี่ยนไปใช้ `dtolnay/rust-toolchain` และอัปเกรด Action อื่นๆ เป็น v4

### Suggested Action
ใช้ `dtolnay/rust-toolchain` แทน `actions-rs` เสมอในโปรเจกต์ Rust

### Metadata
- ที่มา: ข้อผิดพลาด (CI Warnings)
- ไฟล์ที่เกี่ยวข้อง: .github/workflows/*.yml
- แท็ก: github-actions, rust, nodejs-24

---

## [LRN-20260420-002] insight - Gemini CLI Policy Standard

**บันทึกเมื่อ**: 2026-04-20T02:15:00Z
**ลำดับความสำคัญ**: วิกฤต
**สถานะ**: promoted
**ขอบเขต**: backend

### Summary
การออกแบบระบบสิทธิ์ (Permissions) ให้สอดคล้องกับมาตรฐาน Gemini CLI Policy Engine

### Details
- Gemini CLI ใช้ระบบ Tier (1-5) และ Priority (0-999)
- กฎในระดับ Extension (Tier 2) ไม่ควรมี Priority สูงเกินไป (แนะนำไม่เกิน 300) เพื่อให้ผู้ใช้คุมระบบได้
- ระบบสิทธิ์ควรเป็นแบบ Nested Map (Object-based) เพื่อประสิทธิภาพในการ Lookup

### Suggested Action
ใช้โครงสร้าง `{ "tools": { "tool_name": "decision" } }` แทน Array ในการเก็บนโยบาย

### Metadata
- ที่มา: ข้อเสนอแนะจากผู้ใช้
- ไฟล์ที่เกี่ยวข้อง: src/registry/mod.rs, agents/agents.json
- แท็ก: security, policy-engine, gemini-cli

---

## [LRN-20260420-003] correction - Path Consolidation

**บันทึกเมื่อ**: 2026-04-20T02:30:00Z
**ลำดับความสำคัญ**: ปานกลาง
**สถานะ**: promoted
**ขอบเขต**: config

### Summary
ย้ายไฟล์คอนฟิกและสคีมาจาก `.config/` ไปยัง `config/` เพื่อความชัดเจน

### Details
- `.config/` (hidden) ทำให้เกิดความสับสนกับไฟล์ระบบส่วนตัว
- การใช้โฟลเดอร์ `config/` (visible) สื่อว่าเป็นส่วนประกอบหลักของโปรเจกต์
- การแยกเวอร์ชันในพาธ (เช่น `config/v1.7/`) ช่วยในการจัดการการเปลี่ยนแปลงสคีมา

### Suggested Action
ใช้โฟลเดอร์ `config/` สำหรับไฟล์สคีมาและคอนฟิกมาตรฐานของโปรเจกต์

### Metadata
- ที่มา: ข้อเสนอแนะจากผู้ใช้
- ไฟล์ที่เกี่ยวข้อง: config/
- แท็ก: restructuring, project-organization

---

## [LRN-20260420-004] best_practice - Agent Source Abstraction

**บันทึกเมื่อ**: 2026-04-20T02:45:00Z
**ลำดับความสำคัญ**: สูง
**สถานะ**: promoted
**ขอบเขต**: backend

### Summary
เปลี่ยนการอ้างอิงไฟล์เอเจนต์จาก `file` เป็น `source` object (Polymorphic Source)

### Details
- เดิมใช้ฟิลด์ `file` เก็บชื่อไฟล์ตรงๆ ซึ่งไม่ยืดหยุ่น
- โครงสร้างใหม่ `"source": { "type": "builtin", "path": "..." }` รองรับการนำเข้าจาก Git, URL หรือ Local ในอนาคต
- การเพิ่มฟิลด์ `version` ในระดับเอเจนต์ช่วยในการติดตามวงจรชีวิตของทรัพยากร

### Suggested Action
ใช้ Source Abstraction สำหรับทรัพยากรภายนอกที่อาจมีที่มาหลากหลาย

### Metadata
- ที่มา: การสนทนา
- ไฟล์ที่เกี่ยวข้อง: src/registry/schema.rs, agents/agents.json
- แท็ก: architecture, scalability

---

## [LRN-20260420-005] correction - Git Database Repair

**บันทึกเมื่อ**: 2026-04-20T03:00:00Z
**ลำดับความสำคัญ**: วิกฤต
**สถานะ**: promoted
**ขอบเขต**: infra

### Summary
วิธีแก้ปัญหา Git Database Corrupt (Empty Object) และ Stuck Rebase

### Details
- เกิดปัญหา `error: object file ... is empty` ขวางการทำงานของ Git
- สาเหตุ: ไฟล์ระบบขัดข้องหรือปิดไม่สมบูรณ์
- วิธีแก้: ลบไฟล์ Object ที่เสียใน `.git/objects/` ทิ้ง และรัน `git add .` เพื่อสร้างใหม่จากไฟล์จริงใน Workspace

### Suggested Action
เมื่อเจอ Empty Object ให้ลบทิ้งและสร้างใหม่ หรือใช้ Hard Reset ไปที่ Remote ที่เสถียร

### Metadata
- ที่มา: ข้อผิดพลาด
- ไฟล์ที่เกี่ยวข้อง: .git/
- แท็ก: git-repair, troubleshooting
