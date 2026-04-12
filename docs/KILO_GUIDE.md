# Kilo AI - Prompt Templates & Usage Guide

## 🔥 Quick Start

### 1. ติดตั้ง Kilo CLI
```bash
# จาก root ของโปรเจค
kilo github install

# หรือ manual
curl -fsSL https://kilo.ai/cli/install | bash
```

### 2. ตั้งค่า Secrets ใน GitHub
ไปที่ **Settings → Secrets and variables → Actions** เพิ่ม:
- `KILO_API_KEY` - API key จาก Kilo
- `KILO_ORG_ID` - Organization ID (ถ้ามี)

---

## 💬 Trigger Commands

ใช้ `/kilo` หรือ `/kc` ใน:
- ✅ GitHub Issue comments
- ✅ Pull Request comments
- ✅ Code review comments (specific lines)

---

## 📋 Prompt Templates

### 🔍 Review PR

```
/kilo review this PR

เน้นตรวจสอบ:
- Code quality และ best practices
- Security vulnerabilities
- Performance issues
- Missing test coverage
- TypeScript/Rust type safety

ให้คะแนนจาก 1-10 พร้อมเหตุผล
```

### 🐛 Fix Issue

```
/kilo fix this issue

Requirements:
- สร้าง branch ใหม่จาก main
- เขียน tests ก่อน (TDD)
- แก้ไขโค้ดให้ tests ผ่าน
- Commit ด้วย conventional commit message
- เปิด PR กลับมา

ใช้ Rust patterns ที่มีอยู่ในโปรเจค
```

### 📝 Explain Code

```
/kilo explain this code

อธิบาย:
- Architecture และการออกแบบ
- Data flow
- Error handling
- Performance characteristics

ตอบเป็นภาษาไทย
```

### 🧪 Add Tests

```
/kilo add tests for this file

Requirements:
- Unit tests สำหรับทุก public function
- Edge cases
- Integration tests (ถ้าจำเป็น)
- Coverage >90%

ใช้ mockall สำหรับ mocking
ใช้ serial_test สำหรับ tests ที่ต้องรันตามลำดับ
```

### 🛡️ Security Review

```
/kilo security review

ตรวจสอบ:
- Hardcoded secrets
- Unsafe blocks
- Input validation
- Dependency vulnerabilities
- Permission bypass

รายงานผลในรูปแบบ table พร้อม severity
```

### 📊 Generate Documentation

```
/kilo generate documentation for src/registry/

Requirements:
- Rustdoc สำหรับทุก public API
- README section
- Usage examples
- Inline comments เป็นภาษาไทย
```

### 🔄 Refactor

```
/kilo refactor this code

Goals:
- Reduce complexity
- Improve readability
- Follow project conventions
- Keep tests passing

อย่าเปลี่ยน behavior
```

### 🚀 Performance Optimization

```
/kilo optimize performance

Target:
- Reduce allocations
- Minimize clones
- Use async efficiently
- Add benchmarks

วัดผลก่อน-หลัง
```

---

## 🎯 Custom Prompts สำหรับโปรเจคนี้

### Rust-Specific

```
/kilo

Analyze this Rust code for:
1. Ownership/borrowing issues
2. Unnecessary clones
3. Error handling patterns
4. Async runtime usage
5. Memory efficiency

Suggest improvements with code examples
```

### Conductor Workflow

```
/kilo

ตาม Conductor workflow:
1. อ่าน plan.md และ spec.md
2. ตรวจสอบ task status
3. รัน tests (cargo test)
4. ตรวจสอบ coverage
5. อัปเดต plan.md ถ้า task เสร็จ

Report ผลเป็นภาษาไทย
```

### Commit Message

```
/kilo generate commit message for these changes

ตาม conventional commit:
- type(scope): description
- Types: feat, fix, docs, style, refactor, perf, test, chore, security, conductor
- Max 72 characters

ให้ commit message ที่เหมาะสม + body อธิบายการเปลี่ยนแปลง
```

---

## 🔧 Configuration

### Model Selection

Workflow นี้ใช้ **`qwen/qwen3.6-plus:free`** (ฟรี)

เปลี่ยน model ได้ใน `.github/workflows/kilo.yml`:
```yaml
with:
  model: anthropic/claude-sonnet-4-20250514  # ใช้ ANTHROPIC_API_KEY
  # หรือ
  model: openai/gpt-4o                       # ใช้ OPENAI_API_KEY
  # หรือ
  model: qwen/qwen3.6-plus:free              # ใช้ KILO_API_KEY
```

### Environment Variables

เพิ่ม secrets ตามต้องการ:
```bash
# ตั้งค่าใน GitHub Settings → Secrets
KILO_API_KEY=your_key
KILO_ORG_ID=your_org_id
ANTHROPIC_API_KEY=your_key    # ถ้าใช้ Claude
OPENAI_API_KEY=your_key       # ถ้าใช้ GPT
```

---

## 📝 ตัวอย่างการใช้งานจริง

### 1. Review PR พร้อมขอ changes

```
/kc review this PR and fix any issues you find
```

Kilo จะ:
- อ่าน PR diff ทั้งหมด
- ตรวจสอบ code quality
- แก้ไขปัญหาที่พบ
- Commit กลับไป PR เดิม

### 2. Explain specific code lines

เลือก code lines ใน PR → comment:
```
/kc explain this function
```

Kilo จะ:
- อ่าน file + line ที่เลือก
- อธิบาย logic
- ให้ข้อเสนอแนะ

### 3. Fix issue จาก comment

```
/kilo fix this bug

Steps to reproduce:
1. Run cargo test
2. Test X fails
3. Expected: pass

Fix the issue and push changes
```

### 4. Generate tests

```
/kc add comprehensive tests for src/registry/schema.rs

Cover:
- All public functions
- Error cases
- Edge cases
- Serialization/deserialization
```

---

## 🚨 ข้อควรระวัง

1. **Permissions**: Workflow มีสิทธิ์ write contents + PRs
2. **Review ก่อน merge**: Kilo อาจสร้าง changes ที่ต้องตรวจสอบ
3. **Rate limits**: qwen/qwen3.6-plus:free อาจมี rate limits
4. **Context limits**: Kilo อ่านได้เฉพาะ files ใน repo + PR diff

---

## 🎓 Tips

### ใช้ /kc สั้นๆ สำหรับ quick tasks
```
/kc fix this
/kc explain
/kc add tests
```

### ใช้ /kilo สำหรับ detailed prompts
```
/kilo refactored this code using the repository patterns.
Please:
1. Verify the changes match the spec
2. Check for edge cases
3. Suggest improvements
```

### Manual trigger
ไปที่ **Actions → kilo → Run workflow** ใส่ prompt เองได้เลย

---

*Last updated: 2026-04-12*
