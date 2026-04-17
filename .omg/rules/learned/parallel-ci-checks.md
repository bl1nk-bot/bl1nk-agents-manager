---
name: parallel-ci-checks
description: >
  รัน fmt, clippy, test พร้อมกันแบบ parallel
  ประหยัดเวลา 2-3x เทียบกับรันตามลำดับ
globs:
  - "scripts/parallel-check.sh"
  - "Makefile"
---

# Parallel CI Checks

## วิธีใช้

```bash
# เร็วกว่า make all-check
make parallel

# แสดง output realtime
make parallel-verbose

# รันเฉพาะตัวใดตัวหนึ่ง
make fmt-only
make clippy-only
make test-only
```

## ทำไมต้อง parallel

- `cargo fmt` ไม่ขึ้นกับ `cargo clippy` หรือ `cargo test`
- รันพร้อมกันได้ → ประหยัดเวลา
- เก็บ logs ไว้ที่ `target/check-logs/`

## Scripts ที่เกี่ยวข้อง

- `scripts/parallel-check.sh` - script หลัก
- `scripts/bumpversion.sh` - bump version + tag
- `scripts/generate-changelog.sh` - สร้าง changelog จาก commits
- `scripts/commitlint.sh` - ตรวจสอบ commit messages
