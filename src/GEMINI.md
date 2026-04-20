# Agent Management Core

ส่วนนี้คือหัวใจหลักในการจัดการเอเจนต์ของระบบ:
- **Registry**: จัดการสถานะและการลงทะเบียนเอเจนต์ที่โหลดมาจากหน่วยความจำ (v1.7.5.1 รองรับ Universal Tools และ Source Abstraction)
- **Router**: ใช้ตรรกะอัจฉริยะ (Dynamic Weighting) ในการเลือกเอเจนต์ที่เหมาะสมที่สุดโดยอิงตามคะแนนความเชื่อใจ (Trust Score)
- **Executor**: รับผิดชอบวงจรชีวิตงาน พร้อมระบบความปลอดภัย (Policy Enforcement) และการลองใหม่ (Retry)
- **Creator**: สร้างเอเจนต์ใหม่ตามมาตรฐานสิทธิ์แบบลำดับชั้น (Tiered Permissions)

## 🏛️ สถาปัตยกรรมและการตั้งค่า (Configuration Standards)

1. **Path Consolidation**: ใช้โฟลเดอร์ `config/` (ไม่ซ่อน) สำหรับสคีมาและคอนฟิกหลักของโปรเจกต์ เพื่อความชัดเจนและเข้าถึงง่าย
2. **Standardized Schemas**: สคีมาต้องระบุหน้าที่ชัดเจน (capability, policy, app) พร้อมวันที่และเวอร์ชันกำกับ
3. **Source Abstraction**: การระบุที่มาของเอเจนต์ต้องใช้ Object `source` แทนพาธตรงๆ เพื่อรองรับ Git/Remote ในอนาคต

## 🛡️ นโยบายความปลอดภัย (Policy Standards)

- **Hierarchy**: ยึดตามมาตรฐาน Gemini CLI (Tier 1-5, Priority 0-999)
- **Extension Etiquette**: กฎในระดับ Extension (Tier 2) ต้องมี Priority **ไม่เกิน 300** เพื่อเปิดพื้นที่ให้ Workspace (Tier 3) และผู้ใช้ (Tier 4)
- **Nested Tools Map**: จัดเก็บสิทธิ์ในรูปแบบ Map `{ "tools": { "tool_name": "decision" } }` เพื่อประสิทธิภาพและความโปร่งใส

**คำแนะนำสำหรับเอเจนต์:**
- เมื่อแก้ไข Logic การรันงาน ให้ตรวจสอบความสอดคล้องกับ `src/mcp/` และ `config/`
- เมื่อเพิ่มความสามารถใหม่ ให้ตรวจสอบที่ `AgentConfig` ใน `src/config.rs` และอัปเดตสคีมาใน `config/v1.7/` เสมอ
