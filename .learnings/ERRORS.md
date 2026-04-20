# Errors

Failures, exceptions, and unexpected behaviors captured during development.

---

## [ERR-20260420-001] Git Corrupt Object (Empty Object)

**บันทึกเมื่อ**: 2026-04-20T01:30:00Z
**ลำดับความสำคัญ**: สูง
**สถานะ**: resolved
**ขอบเขต**: infra

### Summary
ไฟล์ Object ใน Git ว่างเปล่า (Empty Object) ขัดขวางการ Commit และ Rebase

### Error
```
error: object file .git/objects/c6/5273bbab00161e67e15f10bf1f0f098ab2ad29 is empty
fatal: unable to read c65273bbab00161e67e15f10bf1f0f098ab2ad29
```

### Context
- พยายามรัน `git commit` หลังจากแก้ไขไฟล์จำนวนมาก
- เครื่องอาจมีการค้างหรือระบบไฟล์ขัดข้องชั่วคราว

### Suggested Fix
1. ระบุไฟล์ที่เสียด้วย `git fsck --full`
2. ลบไฟล์ที่เสียออกจาก `.git/objects/` โดยตรง
3. รัน `git add .` เพื่อสร้าง Object ใหม่จากไฟล์ใน Workspace

### Metadata
- ทำซ้ำได้: ไม่
- ไฟล์ที่เกี่ยวข้อง: .git/objects/
- ดูเพิ่มเติม: LRN-20260420-005

---

## [ERR-20260420-002] CI Failure: Submodule Missing

**บันทึกเมื่อ**: 2026-04-20T04:00:00Z
**ลำดับความสำคัญ**: สูง
**สถานะ**: resolved
**ขอบเขต**: infra

### Summary
GitHub Actions รันไม่ผ่านเพราะหาไฟล์ใน Submodule ไม่เจอ

### Error
```
error: failed to read `vendor/bl1nk-keyword-validator/core/Cargo.toml`
Caused by: No such file or directory (os error 2)
```

### Context
- รัน `cargo update` หรือ `cargo build` บน GitHub Actions
- `actions/checkout` ไม่ได้ดึง submodule ลงมาด้วยโดยปริยาย

### Suggested Fix
อัปเดตไฟล์ YAML ใน `.github/workflows/` ให้ใช้ `submodules: recursive` ในขั้นตอน checkout

### Metadata
- ทำซ้ำได้: ใช่
- ไฟล์ที่เกี่ยวข้อง: .github/workflows/*.yml
- แท็ก: ci-cd, git-submodules
