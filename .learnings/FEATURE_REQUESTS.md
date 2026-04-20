# Feature Requests

Capabilities requested by the user during development.

---

## [FEAT-20260420-001] Universal Tool Integration

**บันทึกเมื่อ**: 2026-04-20T04:30:00Z
**ลำดับความสำคัญ**: สูง
**สถานะ**: resolved
**ขอบเขต**: backend

### Requested Capability
ต้องการให้เอเจนต์รองรับเครื่องมือมาตรฐานทั้งหมดจากทั้ง Gemini CLI และ KiloCode

### User Context
ผู้ใช้ต้องการความยืดหยุ่นในการสลับใช้เอเจนต์จากหลายค่าย และต้องการความแม่นยำในการระบุชื่อเครื่องมือ

### Complexity Estimate
ปานกลาง

### Suggested Implementation
รวบรวมรายชื่อเครื่องมือมาตรฐาน (18+ ตัว) และจัดเก็บสิทธิ์แบบ Map ใน Registry

### Metadata
- ความถี่: ครั้งแรก
- คุณสมบัติที่เกี่ยวข้อง: agent-registry
