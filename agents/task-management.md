---
name: task-management
description: เปลี่ยนสถานะจาก "🔄" เป็น "✅"  เพิ่มวันที่เสร็จสิ้น
mode: subagent
tool:
- AskUserQuestion
- ExitPlanMode
- Glob
- Grep
- ListFiles
- ReadFile
- SaveMemory
- Skill
- TodoWrite
- WebFetch
- WebSearch
- WriteFile
---

# Task Management Rules

## Task Completion Tracking

### เมื่องานเสร็จสิ้น ต้องอัปเดต task list ทันที

1. **อัปเดตสถานะในไฟล์ที่เกี่ยวข้อง:**
   - เปลี่ยนสถานะจาก "🔄" เป็น "✅"
   - เพิ่มวันที่เสร็จสิ้น
   - บันทึกผลลัพธ์ที่ได้

2. **อัปเดตในเอกสารหลัก:**
   - `DEVELOPMENT_ROADMAP.md` (mdc:chonost-manuscript-os/DEVELOPMENT_ROADMAP.md) - อัปเดตสถานะ phase
   - `CURRENT_STATUS_SUMMARY.md` (mdc:chonost-manuscript-os/CURRENT_STATUS_SUMMARY.md) - อัปเดตสถานะระบบ
   - `README_NEW.md` (mdc:README_NEW.md) - อัปเดตฟีเจอร์ที่เสร็จสิ้น

3. **บันทึกใน Context Manager:**
   - อัปเดต project status ใน `context_manager.py` (mdc:src/core/context_manager.py)
   - บันทึกความคืบหน้าใน database

### รูปแบบการอัปเดต

```markdown
## ✅ Completed Tasks

### [วันที่] - Task Name
- **Status:** ✅ Completed
- **Result:** ผลลัพธ์ที่ได้
- **Files Modified:** ไฟล์ที่แก้ไข
- **Next Steps:** ขั้นตอนต่อไป

## 🔄 In Progress

### Current Task
- **Status:** 🔄 In Progress
- **Started:** [วันที่เริ่ม]
- **Expected Completion:** [วันที่คาดว่าจะเสร็จ]
```

## Task Categories

### 🚀 High Priority

- ระบบที่ต้องทำงานได้ก่อน
- การแก้ไข bugs วิกฤต
- การตั้งค่า Azure/API

### 📚 Documentation

- อัปเดต README
- สร้าง API documentation
- บันทึกการเปลี่ยนแปลง

### 🔧 Development

- การพัฒนา features ใหม่
- การ refactor โค้ด
- การเพิ่ม tests

### 🧪 Testing

- ทดสอบระบบ
- ทดสอบ API endpoints
- ทดสอบ AI agents

## การจัดการ Context

### อัปเดต Context Manager

```python
# ใน context_manager.py
context_manager.update_project_status(
    component="ai_system",
    status="completed",
    details={
        "azure_models": "working",
        "local_models": "disabled",
        "embedding_system": "active"
    }
)
```

### อัปเดต Prompt Templates

- ตรวจสอบว่า prompt templates สอดคล้องกับสถานะปัจจุบัน
- อัปเดต context injection ตามความคืบหน้า

## การติดตามความคืบหน้า

### ไฟล์ที่ต้องอัปเดตเมื่องานเสร็จ

1. `DEVELOPMENT_ROADMAP.md` (mdc:chonost-manuscript-os/DEVELOPMENT_ROADMAP.md)
2. `CURRENT_STATUS_SUMMARY.md` (mdc:chonost-manuscript-os/CURRENT_STATUS_SUMMARY.md)
3. `README_NEW.md` (mdc:README_NEW.md)
4. `src/core/context_manager.py` (mdc:src/core/context_manager.py)
5. `run_chonost.py` (mdc:run_chonost.py) - อัปเดต health check status

### การบันทึกผลลัพธ์

- บันทึก error messages และการแก้ไข
- บันทึก performance metrics
- บันทึก user feedback
- บันทึก cost estimates สำหรับ Azure models
