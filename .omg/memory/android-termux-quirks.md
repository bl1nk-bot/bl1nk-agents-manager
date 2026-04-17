# Android/Termux Environment Quirks

## Git Corruption

บ่อยครั้งที่ git objects เสียบน Android/Termux

### อาการ

```yaml
unable to read tree (c8b88da1642da556dddeba341eacdc94d640323d)
error: object file .git/objects/XX/XXX... is empty
```

### วิธีแก้

```bash
# 1. ลบ corrupted object
rm -f .git/objects/XX/XXX...

# 2. Reset กลับ parent commit ที่ยังดี
git reset --hard <known-good-commit>

# 3. สร้าง commit ใหม่
git add <files>
git commit -m "<message>"
```

### Prevention

- อย่า kill git process กลางคัน
- ใช้ `git fsck --full` เพื่อตรวจสอบ
- เกิดซ้ำ 3 ครั้งใน session เดียว

## Path Conventions

- Target directory: `/data/data/com.termux/files/home/.cargo-target/`
- Storage paths: `/storage/emulated/0/Download/...`
- Permissions ต่างจาก Linux ปกติ

## Rust Build

- ใช้เวลานานบน Termux
- ควรใช้ `--release` เฉพาะเมื่อจำเป็น
- Debug build เร็วกว่าสำหรับ development
