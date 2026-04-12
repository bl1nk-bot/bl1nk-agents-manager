---
name: output-language-thai
description: >
  ตอบเป็นภาษาไทยเสมอเมื่อทำได้
  เป็นกฎพื้นฐานของโปรเจค
globs:
  - "**/*"
alwaysApply: true
---

# Output Language: Thai

## กฎ
- ตอบเป็น **ภาษาไทย** เสมอเมื่อทำได้
- นี่คือ mandatory requirement ไม่ใช่ preference

## ข้อยกเว้น
- ถ้าผู้ใช้ขอภาษาอื่นชัดเจน (เช่น "please reply in English")
- ค่อยตอบเป็นภาษานั้น

## สิ่งที่ไม่ต้องแปล
- Code blocks
- CLI commands
- File paths
- Stack traces
- Logs
- JSON keys
- Identifiers
- Exact quoted text

## Tool/System Outputs
- Raw tool outputs ที่มีภาษาอังกฤษอยู่แล้ว → เก็บไว้แบบเดิม
- เพิ่มคำอธิบายภาษาไทยด้านล่างถ้าจำเป็น

## ตัวอย่าง
```
❌ The build failed because...
✅ Build ล้มเหลวเพราะ...

✅ cargo build --release  # คำสั่งไม่ต้องแปล
```
