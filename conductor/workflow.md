# Project Workflow

## Guiding Principles

1. **The Plan is the Source of Truth:** All work must be tracked in `plan.md`
2. **The Tech Stack is Deliberate:** Changes to the tech stack must be documented in `tech-stack.md` *before* implementation
3. **Test-Driven Development (STRICT):** Write tests BEFORE implementation - ALWAYS. No exceptions.
4. **High Code Coverage:** Target **>90%** code coverage for all modules
5. **User Experience First:** Every decision must prioritize user experience
6. **Non-Interactive & CI-Aware:** Prefer non-interactive commands. Use `CI=true` for watch-mode tools to ensure single execution.
7. **TODO Required:** ทุกงานที่ยังไม่เสร็จต้องมี `// TODO: <รายละเอียด>` พร้อมอธิบาย API ที่ต้องทำ
8. **todo.md is Working Document:** สร้างและอัปเดต `todo.md` เสมอเพื่อเป็นเอกสารอ้างอิง, รายงานการทำงาน, และ checklist ติดตามสถานะ - แต่ไม่ใช่หลักฐานยืนยันความสำเร็จ (tests เป็นหลักฐานจริง)
9. **Thai Comments:** คอมเมนต์โค้ดต้องเป็นภาษาไทย อธิบายเหตุผล (why) ไม่ใช่แค่สิ่งที่ทำ (what)

## Task Lifecycle

### Standard Task Workflow

1. **Read Context:** อ่าน `todo.md` และ `plan.md` เพื่อเข้าใจสถานะและขอบเขตงาน

2. **Select Task:** เลือก task ถัดจาก `plan.md` ตามลำดับ

3. **Mark In Progress:** แก้ไข `plan.md` เปลี่ยนจาก `[ ]` เป็น `[~]`

4. **Update todo.md:** อัปเดตสถานะใน `todo.md` ให้สอดคล้อง

5. **Write Failing Tests (Red Phase):**
   - สร้าง test file สำหรับฟีเจอร์หรือ bug fix
   - เขียน test ที่ชัดเจนตาม acceptance criteria
   - **CRITICAL:** รัน test และยืนยันว่า **fail** ตามที่คาดหวัง
   - **ห้ามเดินหน้า** จนกว่าจะมี failing tests
   - ทดสอบทั้ง success และ **edge cases**

6. **Implement to Pass Tests (Green Phase):**
   - เขียนโค้ดน้อยที่สุดที่ทำให้ test ผ่าน
   - รัน test suite อีกครั้งเพื่อยืนยันว่า **ผ่านทั้งหมด**

7. **Refactor (Optional but Recommended):**
   - ปรับปรุงโค้ดให้อ่านง่าย ลบ duplication
   - รัน test อีกครั้งเพื่อยืนยันว่ายังผ่าน

8. **Verify Coverage:**
   ```bash
   cargo test --all-features -- --test-threads=1
   cargo tarpaulin --out Html --output-dir coverage
   ```
   **Target: >90% coverage** สำหรับโค้ดใหม่

9. **Document Deviations:** หาก implementation ต่างจาก tech-stack.md:
   - **หยุด** implementation
   - อัปเดต `tech-stack.md` พร้อมเหตุผล
   - ใส่วันที่และคำอธิบาย
   - ทำงานต่อ

10. **Commit Code Changes:**
    - Stage ไฟล์ที่เกี่ยวข้อง
    - Proposed commit message เช่น `feat(mcp): เพิ่ม delegate_task tool พร้อม proposal system`
    - ทำการ commit

11. **Attach Task Summary (Both Methods):**
    - **Step 11.1: Get Commit Hash:** `git log -1 --format="%H"`
    - **Step 11.2: Draft Note (Thai):** สรุปงานที่เสร็จ รวมถึงไฟล์ที่สร้าง/แก้ไข และเหตุผล
    - **Step 11.3: Attach Git Note:**
      ```bash
      git notes add -m "<สรุปงานภาษาไทย>" <commit_hash>
      ```
    - **Step 11.4: Also in Commit Message Body:** ใส่สรุปเดียวกันใน commit message

12. **Get and Record Task Commit SHA:**
    - **Step 12.1: Update Plan:** แก้ไข `plan.md` จาก `[~]` เป็น `[x]` พร้อม commit hash 7 ตัวแรก
    - **Step 12.2: Update todo.md:** อัปเดต checklist ให้สอดคล้อง
    - **Step 12.3: Write Files:** บันทึกการเปลี่ยนแปลง

13. **Commit Plan Update:**
    - Stage `plan.md` และ `todo.md`
    - Commit ด้วย message เช่น `conductor(plan): Mark task 'เพิ่ม BM25 routing' as complete`

---

### Phase Completion Verification and Checkpointing Protocol

**Trigger:** ทำงานทันทีเมื่อ task ที่เสร็จเป็นจุดจบของ phase ใน `plan.md`

1. **Announce Protocol Start:** แจ้ง user ว่า phase เสร็จและเริ่ม verification protocol

2. **Ensure Test Coverage for Phase Changes:**
   - **Step 2.1: Determine Scope:** อ่าน `plan.md` หา checkpoint SHA ของ phase ก่อนหน้า (ถ้าไม่มี = ทุก commit แรก)
   - **Step 2.2: List Changed Files:** `git diff --name-only <prev_checkpoint_sha> HEAD`
   - **Step 2.3: Verify/Create Tests:** สำหรับแต่ละไฟล์โค้ดที่เปลี่ยน:
     - ขยายไฟล์ config/docs (`.json`, `.md`, `.toml`)
     - ตรวจสอบว่ามี test file สอดคล้อง
     - ถ้าไม่มี **ต้องสร้างทันที** โดยดู pattern จาก test file อื่นในโปรเจค
     - Test ต้อง validate functionality ตาม task ใน `plan.md`

3. **Execute Automated Tests with Debugging:**
   - **ประกาศคำสั่งก่อนรัน:** "จะรัน test suite ด้วยคำสั่ง: `cargo test --all-features`"
   - รันคำสั่ง
   - ถ้า fail → debug และ fix ได้สูงสุด **2 ครั้ง**
   - ถ้ายัง fail หลัง 2 ครั้ง → **หยุด** และรายงาน user เพื่อขอคำแนะนำ

4. **Propose Manual Verification Plan:**
   - วิเคราะห์ `product.md`, `product-guidelines.md`, `plan.md` เพื่อกำหนด goal
   - สร้าง step-by-step plan สำหรับ user ตรวจสอบ:

   **For Backend/CLI Change:**
   ```
   Test ผ่านแล้ว กรุณาตรวจสอบด้วยมือตามขั้นตอน:

   **ขั้นตอนตรวจสอบ:**
   1. **Build โปรเจค:** `cargo build --release`
   2. **รัน binary:** `./target/release/bl1nk-agents-manager --help`
   3. **ยืนยันว่าเห็น:** รายการ commands ใหม่ที่เพิ่มเข้ามา
   4. **Test edge case:** ลองใส่ input ที่ไม่ถูกต้อง ควรได้ error message ที่ชัดเจน
   ```

5. **Await Explicit User Feedback:**
   - ถาม user: "**นี่ตรงตามที่ต้องการหรือไม่? กรุณายืนยัน yes หรือแจ้งสิ่งที่ต้องแก้**"
   - **หยุดรอ** จนกว่า user จะตอบ yes

6. **Create Checkpoint Commit:**
   - Stage ไฟล์ทั้งหมด (ถ้าไม่มีเปลี่ยนแปลง = empty commit)
   - Commit ด้วย message เช่น `conductor(checkpoint): Checkpoint end of Phase 1 - BM25 Routing`

7. **Attach Verification Report (Git Notes):**
   - สร้างรายงานรวมถึง: test command, manual steps, user confirmation
   - Attach ด้วย `git notes`

8. **Get and Record Phase Checkpoint SHA:**
   - **Step 8.1:** `git log -1 --format="%H"` เพื่อหา checkpoint hash
   - **Step 8.2:** อ่าน `plan.md` เพิ่ม `[checkpoint: <sha7>]` ที่ header ของ phase
   - **Step 8.3:** บันทึกไฟล์

9. **Commit Plan Update:**
   - Commit ด้วย message เช่น `conductor(plan): Mark phase 'BM25 Routing' as complete`

10. **Announce Completion:** แจ้ง user ว่า phase เสร็จสมบูรณ์ พร้อม checkpoint

---

### Quality Gates (Must Pass Before Marking Complete)

- [ ] Test ทั้งหมดผ่าน (ไม่มี fail)
- [ ] Code coverage **>90%** สำหรับโค้ดใหม่
- [ ] โค้ดตาม style guide ใน `code_styleguides/rust.md`
- [ ] Public functions มี rustdoc ครบถ้วน
- [ ] ไม่มี clippy warnings (`cargo clippy -- -D warnings`)
- [ ] `cargo fmt --check` ผ่าน
- [ ] ทดสอบบน **Android/Termux** (ถ้าเกี่ยวข้อง)
- [ ] BM25 search quality benchmarks ผ่าน (ถ้าเพิ่ม search feature)
- [ ] Edge cases ถูกทดสอบและบันทึกผล
- [ ] `todo.md` อัปเดตแล้ว
- [ ] ไม่มี security vulnerabilities

---

## Development Commands (Rust/bl1nk-agents-manager)

### Setup
```bash
# ติดตั้ง dependencies
cargo fetch

# ติดตั้ง cargo-tarpaulin สำหรับ coverage
cargo install cargo-tarpaulin

# Verify build
cargo build --all-features
```

### Daily Development
```bash
# Build (debug)
cargo build

# Build (release)
cargo build --release

# Run tests
cargo test --all-features

# Run single test
cargo test test_name_here

# Run tests with output
cargo test -- --nocapture

# Lint + check
cargo clippy -- -D warnings

# Format code
cargo fmt

# Check without building
cargo check

# Run with config
cargo run -- --config config.toml
```

### Coverage
```bash
# Generate coverage report
cargo tarpaulin --out Html --output-dir coverage

# Generate coverage (Termux/Android compatible)
cargo test --all-features -- --test-threads=1
```

### Before Committing
```bash
# Full pre-commit check
cargo fmt --check && cargo clippy -- -D warnings && cargo test --all-features
```

---

## Testing Requirements

### Unit Testing
- ทุก module ต้องมี tests สอดคล้อง
- ใช้ `#[cfg(test)]` mod tests
- Mock external dependencies ด้วย `mockall`
- ทดสอบทั้ง success และ failure cases
- ใช้ `serial_test` สำหรับ tests ที่ต้องรันตามลำดับ

### Integration Testing
- ทดสอบ end-to-end MCP protocol flows
- ทดสอบ agent routing และ rate limiting
- ทดสอบ config parsing และ validation
- ทดสอบ CLI delegation workflow

### Edge Case Testing
- Input validation (empty, null, malformed)
- Rate limit boundary conditions
- Concurrent access to shared state
- Error recovery and graceful degradation
- Config file missing or corrupted

### Mobile/Termux Testing
- ทดสอบบน Android/Termux environment
- ตรวจสอบ path resolution (`/data/data/com.termux/...`)
- ทดสอบ file permissions และ access
- ตรวจสอบ memory usage จำกัด
- ทดสอบ cross-compilation จาก dev machine ไป Termux

### BM25 Search Testing
- เพิ่ม search quality benchmarks เมื่อใช้ BM25
- ทดสอบ search accuracy กับ test corpus
- ทดสอบ performance (latency <100ms)
- ทดสอบ tokenization ต่างๆ (BPE, WordPiece, whitespace)
- วัด precision/recall กับ ground truth queries

---

## Code Review Process

### Self-Review Checklist
ก่อนขอ review:

1. **Functionality**
   - ฟีเจอร์ทำงานตาม spec
   - Edge cases จัดการครบ
   - Error messages ชัดเจนและเป็นประโยชน์

2. **Code Quality**
   - ตาม style guide (`rust.md`)
   - DRY principle
   - ตัวแปร/ฟังก์ชันชื่อชัดเจน
   - คอมเมนต์ภาษาไทยพร้อมเหตุผล

3. **Testing**
   - Unit tests ครบถ้วน
   - Integration tests ผ่าน
   - Coverage >90%
   - Edge cases ถูกทดสอบ

4. **Security**
   - ไม่มี hardcoded secrets
   - Input validation ครบ
   - Permission sandboxing ทำงาน

5. **Performance**
   - Async operations เหมาะสม
   - No unnecessary allocations
   - Rate limiting ทำงาน

6. **Documentation**
   - `todo.md` อัปเดตแล้ว
   - `plan.md` อัปเดตแล้ว
   - Rustdoc สำหรับ public API
   - TODO comments สำหรับงานค้าง

---

## Commit Guidelines

### Message Format
```
<type>(<scope>): <description>

<สรุปงานภาษาไทย - รายละเอียดการเปลี่ยนแปลง>

ไฟล์ที่สร้าง/แก้ไข:
- src/xxx.rs: รายละเอียด
- tests/xxx.rs: รายละเอียด
```

### Types
- `feat`: ฟีเจอร์ใหม่
- `fix`: แก้ bug
- `docs`: เอกสารเท่านั้น
- `style`: Formatting, semicolons, etc.
- `refactor`: Refactor ไม่แก้ bug ไม่เพิ่ม feat
- `test`: เพิ่ม/แก้ไข tests
- `chore`: Maintenance tasks
- `conductor`: การจัดการ conductor files

### Examples
```bash
git commit -m "feat(routing): เพิ่ม BM25 keyword matching

เพิ่มระบบค้นหาด้วย BM25 สำหรับ routing agent
แทนที่ keyword matching แบบเดิม

ไฟล์ที่สร้าง/แก้ไข:
- src/agents/router.rs: เพิ่ม BM25 scoring
- tests/router_test.rs: ทดสอบ routing accuracy"

git commit -m "fix(config): แก้ rate limit parsing สำหรับ TOML nested tables"
git commit -m "test(agents): เพิ่ม integration tests สำหรับ delegate_task"
git commit -m "conductor(plan): Mark task 'เพิ่ม BM25 routing' as complete"
```

---

## Definition of Done

Task ถือว่าเสร็จเมื่อ:

1. ✅ โค้ด implement ตาม specification
2. ✅ Tests เขียนและผ่านทั้งหมด (รวม edge cases)
3. ✅ Code coverage >90% สำหรับโค้ดใหม่
4. ✅ Rustdoc ครบสำหรับ public functions
5. ✅ ผ่าน `cargo clippy -- -D warnings` และ `cargo fmt --check`
6. ✅ ทดสอบบน mobile/Termux (ถ้าเกี่ยวข้อง)
7. ✅ `plan.md` และ `todo.md` อัปเดตแล้ว
8. ✅ Commit พร้อม message และ git note
9. ✅ **user ยืนยันว่าทำงานได้ตามที่ต้องการ** (สำคัญที่สุด)
10. ✅ TODO comments สำหรับงานที่ยังไม่เสร็จ พร้อมรายละเอียด API

**หมายเหตุ:** `todo.md` เป็นเครื่องมือติดตาม แต่ **หลักฐานยืนยันความสำเร็จคือ tests ที่ผ่าน** และ **user confirmation** ว่าทำงานได้ตามต้องการ หากทำไม่ได้หรือ user ยังไม่มั่นใจ ต้องทำต่อไปจนกว่าจะแน่ใจ

---

## Emergency Procedures

### Critical Bug in Production
1. สร้าง hotfix branch จาก main
2. เขียน failing test สำหรับ bug
3. Implement minimal fix
4. Test รวมถึง edge cases
5. Deploy ทันที
6. บันทึกใน `plan.md` และ `todo.md`

### Data Loss
1. หยุด write operations ทั้งหมด
2. Restore จาก backup ล่าสุด
3. ตรวจสอบ data integrity
4. บันทึก incident
5. อัปเดต backup procedures

### Security Breach
1. Rotate secrets ทันที
2. ตรวจสอบ access logs
3. Patch vulnerability
4. แจ้ง affected users
5. อัปเดต security procedures

---

## Deployment Workflow

### Pre-Deployment Checklist
- [ ] Tests ทั้งหมดผ่าน
- [ ] Coverage >90%
- [ ] ไม่มี clippy warnings
- [ ] ทดสอบบน Termux (ถ้าต้อง)
- [ ] BM25 benchmarks ผ่าน (ถ้าใช้ search)
- [ ] Environment variables config พร้อม
- [ ] Backup created

### Deployment Steps
1. Merge feature branch ไป main
2. Tag release ด้วย version
3. Build release binary
4. ทดสอบ critical paths
5. Monitor errors

### Post-Deployment
1. Monitor logs
2. ตรวจสอบ error rate
3. เก็บ user feedback
4. วางแผน iteration ถัดไป

---

## Continuous Improvement

- รีวิว workflow ทุกสัปดาห์
- อัปเดตตาม pain points
- บันทึก lessons learned
- Optimize สำหรับ user happiness
- Keep it simple and maintainable
- **ซื่อสัตย์ต่อสถานะงาน** - รายงานตามจริง
