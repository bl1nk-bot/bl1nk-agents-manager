---
name: bumpversion-and-changelog
description: >
  ใช้ scripts สำหรับ bump version และสร้าง changelog อัตโนมัติ
globs:
  - "scripts/bumpversion.sh"
  - "scripts/generate-changelog.sh"
---

# Bumpversion + Changelog Automation

## Bump Version
```bash
# Patch (0.1.0 -> 0.1.1)
make bump-patch

# Minor (0.1.0 -> 0.2.0)
make bump-minor

# Major (0.1.0 -> 1.0.0)
make bump-major

# Dry run - ดูว่าจะทำอะไรแต่ไม่แก้ไข
./scripts/bumpversion.sh --dry-run patch

# ตั้ง version โดยตรง
./scripts/bumpversion.sh 2.0.0
```

## Generate Changelog
```bash
# สร้างจาก 50 commits ล่าสุด
make changelog

# กำหนดจำนวน commits
./scripts/generate-changelog.sh --max 100

# JSON output
./scripts/generate-changelog.sh --json
```

## สิ่งที่เกิดขึ้น
bumpversion.sh:
1. อ่าน version ปัจจุบันจาก Cargo.toml
2. คำนวณ version ใหม่
3. อัปเดต Cargo.toml + Cargo.lock
4. Commit อัตโนมัติ
5. สร้าง git tag (annotated)

generate-changelog.sh:
1. อ่าน commits จาก git log
2. จัดกลุ่มตาม conventional commit types
3. สร้าง CHANGELOG.md
