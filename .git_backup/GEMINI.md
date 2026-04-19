# 🤖 GEMINI Context (bl1nk-agents)

**Project Identifier:** `bl1nk-agents` (Core Agent Management)
**Vendor Reference:** `bl1nk-keyword-validator` (Fuzzy Search & Validation Infrastructure)

---

## 🛡️ Core Mandates (ลำดับความสำคัญสูงสุด)

1. **Stability & Zero Warnings**: งานทั้งหมดต้องไม่มี Warning จาก Clippy และต้องผ่านเทสต์ 100% ก่อนการ Merge หรือ Commit
2. **Standardization**: ยึดตามมาตรฐาน Markdown (markdownlint-cli2) และ Rust (rustfmt) อย่างเคร่งครัด
3. **Atomic Persistence**: การจัดเก็บสถานะเอเจนต์ใน `.omg/state/` ต้องเป็นแบบ JSON Atomic Write เพื่อความปลอดภัยของข้อมูล
4. **Context Management**: ให้ความสำคัญกับการจัดเก็บ Context ในรูปแบบ JSON (Atomic) และ Markdown (Archives) เพื่อลดปริมาณ Token และเพิ่มความโปร่งใส
5. **ID-Based Tasks**: การอ้างอิงงานต้องใช้ ID จาก `todo.md` หรือ `taskboard.md` เป็นหลักเพื่อให้การติดตามสถานะแม่นยำ

---

## 🏗️ Project Architecture (Agentic Strategy)

- **Registry System**: จัดการวงจรชีวิตของเอเจนต์ผ่าน `src/registry/`
- **Routing Engine**: ใช้ตรรกะใน `src/main.rs` และ `src/config.rs` เพื่อเลือกเอเจนต์ที่เหมาะสมที่สุดตาม Capability
- **PMCP Gateway**: การสื่อสารต้องผ่านโปรโตคอล Model Context Protocol ใน `src/mcp/`
- **Track-Based Planning**: ใช้ระบบ Track จากโฟลเดอร์ `conductor/` เพื่อวางแผนและตัดสินใจในระดับสถาปัตยกรรม

---

## 📜 Workflow Protocols

1. **Research First**: ก่อนเริ่มงานทุกครั้ง ต้องสำรวจ `todo.md`, `conductor/tracks.md` และ `.omg/state/taskboard.md` เพื่อรับทราบสถานะปัจจุบัน
2. **TDD is Mandatory**: เขียนเทสต์ก่อนเริ่มการแก้ไขโค้ดที่ซับซ้อนในโมดูล `tests/`
3. **Status Sync**: เมื่อจบงานชิ้นใด ต้องอัปเดต `todo.md` และสถานะใน `taskboard.md` ทันที
4. **Modernized CI/CD**: ระบบ CI/CD และระบบป้ายกำกับ (v1.7.1+) ต้องเป็นเวอร์ชันปัจจุบันที่อัปเดตใหม่ใน `.github/`

---

**Last Updated:** 2026-04-18 (v1.7.1)
**Managed by:** Gemini Interactive CLI Agent
