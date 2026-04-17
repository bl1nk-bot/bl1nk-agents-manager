# KiloCode - Code Review Workflow

## 🔍 วิธี Review ด้วย Kilo

### แบบที่ 1: Review ใน GitHub (แนะนำ)

Comment ใน PR หรือ Issue:

```text
/kilo review this PR

Focus on:
- Correctness & Security
- Code quality
- Performance
- Edge cases
```

### แบบที่ 2: Review Local Changes

```bash
# ดูว่ามีการเปลี่ยนแปลงอะไร
git diff

# แล้วใช้ Kilo review
kilo run "review these local changes and provide feedback"
```text
### แบบที่ 3: Review File เฉพาะ
```bash
kilo run "review src/registry/schema.rs for correctness, security, and code quality"
```text
---

## 📋 Review Checklist สำหรับ Kilo

เมื่อสั่ง Kilo review ให้ระบุ focus areas:

### 1. Correctness & Security
```

/kilo review this code

Focus:

- Logic errors และ edge cases
- Null/undefined handling
- Security vulnerabilities
- Error handling gaps
- Type safety issues

```text
### 2. Code Quality
```

/kilo review this code

Focus:

- Code style consistency
- Naming conventions
- Code duplication
- Over-engineering
- Dead code

```text
### 3. Performance
```

/kilo review this code

Focus:

- Performance bottlenecks
- Memory usage
- Inefficient algorithms
- Missing caching opportunities

```text
### 4. Complete Review
```

/kilo do a comprehensive code review

Check:

1. Correctness & Security
2. Code Quality
3. Performance
4. Business logic soundness
5. Module boundaries
6. Hidden side effects

Return findings with:

- File and line reference
- What's wrong
- Why it matters
- Suggested fix
- Severity (Critical/Suggestion/Nice to have)

```text
---

## 🎯 ตัวอย่างการใช้งานจริง

### Review PR
```

/kilo review this PR #123

Provide findings in this format:

- **File:** <path>:<line>
- **Issue:** <description>
- **Impact:** <why it matters>
- **Fix:** <suggestion>
- **Severity:** Critical | Suggestion | Nice to have

```text
### Review Before Commit
```bash
git diff | kilo run "review these changes for bugs and security issues"
```text
### Review Specific Feature
```

/kilo review the registry schema implementation

Check for:

- Proper error handling
- Type safety
- Performance with large datasets
- Edge cases

```

---

## 📊 Severity Levels

| Level | Meaning | Action |
|-------|---------|--------|
| **Critical** | Must fix before merge | Bug, security, data loss |
| **Suggestion** | Recommended improvement | Better pattern, clearer code |
| **Nice to have** | Optional | Minor style tweak |

---

## ❌ สิ่งที่ไม่ต้อง Review

- Pre-existing issues ในโค้ดเดิม (ดูเฉพาะ diff)
- Style/formatting ที่ตรงกับ surrounding code
- Issues ที่ linter/type checker จับได้อัตโนมัติ
- Subjective suggestions ที่ไม่ใช่ปัญหาจริง
- ถ้าไม่แน่ใจว่าเป็นปัญหาไหม → **ไม่ต้องรายงาน**

---

## 🔧 Kilo Commands Summary

| Command |做什么 |
|---------|------|
| `/kilo review this PR` | Review PR ทั้งหมด |
| `/kc review <file>` | Review file เฉพาะ |
| `kilo run "review..."` | Review ใน local machine |
| `/kilo fix <issue>` | แก้ไขปัญหาที่พบ |
| `/kilo explain <code>` | อธิบายโค้ด |

---

*Workflow สำหรับ KiloCode + qwen/qwen3.6-plus:free*
