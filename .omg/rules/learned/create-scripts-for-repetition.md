---
name: create-scripts-for-repetition
description: >
  เพื่อลดงานซ้ำๆ ให้สร้าง scripts หรือเสนอ scripts ให้ผู้ใช้
  ออกแบบ scripts ที่ใช้ซ้ำได้เพื่อให้ workflow ลื่นไหล
globs:
  - "scripts/**/*.sh"
---

# Create Scripts for Repetitive Tasks

## กฎ
เมื่อพบงานที่ทำซ้ำหรือมีแนวโน้มจะทำซ้ำ → สร้าง script

## ตัวอย่างที่สร้างแล้ว
| Script |做什么 |
|--------|------|
| `scripts/bumpversion.sh` | Bump version + git tag |
| `scripts/commitlint.sh` | ตรวจสอบ commit messages |
| `scripts/generate-changelog.sh` | สร้าง CHANGELOG.md |
| `scripts/parallel-check.sh` | รัน fmt+clippy+test พร้อมกัน |
| `scripts/update-security.sh` | Security audit + update deps |
| `scripts/review.sh` | Code review wrapper |

## Pattern การสร้าง Script
1. ระบุงานที่ทำซ้ำ
2. สร้าง bash script ที่ automate งานนั้น
3. เพิ่ม Makefile target
4. ทำให้ script มี --help/--dry-run
5. ใส่สี/emoji output เพื่อ readability
6. เก็บ logs ใน `target/` หรือ temp dir

## ควรเสนอให้ผู้ใช้
- "งานนี้ทำได้ด้วย script ให้สร้างไหม?"
- "ต้องการให้เพิ่ม Makefile target ด้วยไหม?"
- "script นี้รองรับ options อะไรบ้าง?"

## ทำไมสำคัญ
- ลดเวลาของผู้ใช้
- ลดงานของ agent
- ทำให้ workflow เป็นระบบ
- ใช้ซ้ำได้ไม่ต้องทำใหม่ทุกครั้ง
