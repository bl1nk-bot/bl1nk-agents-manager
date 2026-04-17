---
name: git-corruption-recovery
description: >
  Android/Termux มักเกิด git corrupted objects บ่อยครั้ง
  วิธีแก้: ลบ object ที่เสีย, git reset, แล้วสร้าง commit ใหม่
globs:
  - ".git/**"
---

# Git Corruption Recovery (Android/Termux)

## อาการ

```yaml
unable to read tree (c8b88da1642da556dddeba341eacdc94d640323d)
error: object file .git/objects/XX/XXX... is empty
```text
## วิธีแก้
```bash
# 1. ลบ corrupted object
rm -f .git/objects/XX/XXX...

# 2. Reset กลับ parent commit ที่ยังดี
git reset --hard <known-good-commit>

# 3. สร้าง commit ใหม่
git add <files>
git commit -m "<message>"
```

## Prevention

- บ่อยครั้งเกิดจาก storage เปลี่ยนแปลงขณะ git กำลังเขียน
- อย่า kill git process กลางคัน
- ใช้ `git fsck --full` เพื่อตรวจสอบ
