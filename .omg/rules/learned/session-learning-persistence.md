---
name: session-learning-persistence
description: >
  เก็บวิธีแก้ไขและปัญหาที่พบระหว่าง session
  เพื่อป้องกันปัญหาเดิมในอนาคต
globs:
  - ".omg/rules/learned/**"
  - ".omg/MEMORY.md"
alwaysApply: true
---

# Session Learning Persistence

## กฎ
ระหว่างทำงานในแต่ละ session:

1. **บันทึกปัญหาที่พบ** - อะไรผิดพลาด แก้ยังไง
2. **บันทึก workaround** - วิธีแก้เฉพาะหน้าสำหรับ environment quirks
3. **บันทึก user preferences** - สิ่งที่ผู้ใช้ย้ำเตือนหรือแก้ไข
4. **บันทึก patterns** - สิ่งที่อาจเกิดซ้ำในอนาคต

## ที่เก็บ
- `.omg/rules/learned/` - Rules เฉพาะที่ reusable
- `.omg/MEMORY.md` - ข้อมูลสำคัญระดับโปรเจค

## ตัวอย่างสิ่งที่ควรบันทึก
- Git corruption recovery (Android/Termux)
- Kilo workflow configuration quirks
- Rust schemars + DateTime workaround
- User corrected agent behaviors

## สิ่งที่ไม่ต้องบันทึก
- One-time fixes ที่ไม่เกี่ยวข้อง
- Conversational fluff
- สิ่งที่ linter/type checker จับได้อัตโนมัติ

## เมื่อไหร่ควรบันทึก
- เมื่อพบปัญหาที่อาจเกิดซ้ำ
- เมื่อผู้ใช้แก้ไข behavior ของ agent
- เมื่อค้นพบ convention/pattern ใหม่ของโปรเจค
- เมื่อสร้าง script/tool ใหม่ที่ใช้ซ้ำได้
