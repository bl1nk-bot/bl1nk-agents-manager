---
name: conventional-commit-linting
description: >
  Commit messages ต้องเป็น conventional commit format
  type(scope): description
globs:
  - "scripts/commitlint.sh"
  - ".commitlintrc.json"
---

# Conventional Commit Linting

## Format
```
type(scope): description
```

## Types ที่อนุญาต
- `feat` - ฟีเจอร์ใหม่
- `fix` - แก้ไข bug
- `docs` - เอกสาร
- `style` - Formatting (ไม่กระทบ logic)
- `refactor` - Refactor code
- `perf` - ปรับปรุง performance
- `test` - เพิ่ม/แก้ไข tests
- `chore` - Maintenance tasks
- `security` - Security updates
- `conductor` - Conductor management
- `ci` - CI/CD changes
- `build` - Build system changes
- `revert` - Revert previous commit

## Rules
1. Subject line ≤ 72 ตัวอักษร
2. Type ต้องเป็นตัวพิมพ์เล็กเท่านั้น
3. Description ต้องไม่เป็น empty
4. Description ไม่ควรลงท้ายด้วย `.`

## ตัวอย่าง
```
✅ feat(registry): Task 1.1 - Define Unified Registry Schema
✅ fix(kilo): ลบ KILO_ORG_ID - ใช้แค่ KILO_API_KEY เท่านั้น
✅ docs: อัปเดต PROJECT_SUMMARY และ README
❌ Fix bug (type ไม่ตัวเล็ก)
❌ update (ไม่มี colon และ description)
```

## วิธีใช้
```bash
make commitlint          # ตรวจสอบ commit ล่าสุด
make commitlint-range    # ตรวจสอบ 10 commits ล่าสุด
```
