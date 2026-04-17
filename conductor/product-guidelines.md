# Product Guidelines

## 1. Documentation & Comment Standards

### 1.1 Development Documentation (ภาษาไทย)

- **โค้ดคอมเมนต์:** ทุกไฟล์ที่พัฒนาใหม่ ต้องมีคอมเมนต์ภาษาไทยที่อธิบาย:
  - `// TODO: <รายละเอียด>` สำหรับงานที่ยังไม่เสร็จหรือต้องเพิ่มเติม
  - `/** ... */` สำหรับ JSDoc/Rustdoc ที่อธิบาย module, function, param, return
  - คอมเมนต์อธิบาย *เหตุผล* (why) ไม่ใช่แค่ *สิ่งที่ทำ* (what)
- **เอกสารพัฒนา:** สร้างใน `docs/` และเพิ่มลิงก์ใน `README.md`

### 1.2 Commit & Release

- **Conventional Commits:** ใช้ prefix `feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `chore:`
- **Keep a Changelog:** บันทึกการเปลี่ยนแปลงใน `CHANGELOG.md` ตามรูปแบบ keepachangelog.com
- **Per-Task Commits:** commit แยกตาม task ที่เสร็จ ไม่ใช่รวมหลายอย่างใน commit เดียว

### 1.3 Public API Documentation

- **Google Style:** docstrings แบบ Google-style พร้อม examples สำหรับ public functions
- **Examples:** ทุก module สำคัญต้องมี example ใน `examples/`

---

## 2. Code Quality Standards

### 2.1 Linting & Formatting

- **clippy:** ห้ามมี clippy warnings ทุกกรณี
- **rustfmt:** รันทุกครั้งก่อน commit ใช้ config ใน `rustfmt.toml`
- **No unsafe:** ห้ามนำ `unsafe` blocks มาใช้ ยกเว้นมีเหตุผลชัดเจนและผ่าน review

### 2.2 Test Coverage

- **>80% coverage:** ทุกฟีเจอร์ใหม่ต้องมี test coverage อย่างน้อย 80%
- **Test-Driven:** เขียน test ก่อน implement (TDD) สำหรับ logic ที่ซับซ้อน
- **Integration Tests:** มี tests สำหรับ end-to-end flows ใน `tests/`

### 2.3 Development Workflow

- **วางแผนก่อน:** เริ่มจากวิเคราะห์ → สร้าง schema ใน `.config/*` → สร้าง `todo.md` ตามแผน
- **รอคำสั่งเริ่ม:** ไม่เริ่ม implement จนกว่าผู้ใช้สั่ง "เริ่ม"
- **ซื่อสัตย์:** รายงานสถานะตามจริง ไม่บอกว่าเสร็จถ้ายังไม่เสร็จ

---

## 3. CLI User Experience

### 3.1 Output Style

- **Emoji-rich:** ใช้ emoji สำหรับ status indicators:
  - ✅ สำเร็จ, ❌ ผิดพลาด, 🚀 เริ่มทำงาน, 🔍 กำลังค้นหา, ⚠️ คำเตือน
- **Progress Indicators:** แสดง progress bar หรือ spinner สำหรับงานที่ใช้เวลานาน
- **Machine-parseable:** มี flag `--json` สำหรับ output แบบ JSON สำหรับ scripting

### 3.2 User Commands

- **Consistent Format:** ใช้รูปแบบ `/command:subcommand` สำหรับทุกคำสั่ง
- **Help Text:** ทุกคำสั่งต้องมี help text ที่ชัดเจน พร้อม examples
- **Interactive Mode:** ถามยืนยันก่อนทำ destructive actions เสมอ

---

## 4. Error Handling

### 4.1 Error Messages

- **Verbose Context:** ข้อความ error ต้องอธิบายว่าเกิดอะไรขึ้น และแนะนำวิธีแก้
- **Error Codes:** ใช้ error codes แบบ numeric พร้อม lookup ในเอกสาร
- **Recovery Hints:** แนะนำคำสั่งหรือขั้นตอนถัดไปเมื่อเกิด error

### 4.2 Error Recovery

- **Graceful Degradation:** ระบบต้องทำงานต่อได้เมื่อ agent ตัวใดตัวหนึ่งล้มเหลว
- **State Persistence:** บันทึกสถานะก่อน crash เพื่อให้ resume ได้
- **Audit Trail:** บันทึก error log พร้อม correlation ID สำหรับ debug

---

## 5. Security & Permissions

### 5.1 Shell Execution

- **Sandboxing:** จำกัดสิทธิ์ shell commands ที่ agent รันได้
- **Allowlist:** มีรายการ commands ที่อนุญาตเท่านั้น
- **User Confirmation:** ถามก่อนรัน commands ที่มีความเสี่ยง

### 5.2 Configuration

- **Secrets Management:** ห้ามเก็บ API keys หรือ tokens ใน plaintext
- **File Permissions:** ไฟล์ config ต้องมี permission 600 หรือเข้มงวดกว่า
