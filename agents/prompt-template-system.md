## 📌 Project Status (Feb 7, 2026)

Bl1nk Agents Manager is in active development and is not feature‑complete yet.
This repo contains a working extension shell and a Rust core that is being
brought to feature parity with existing TypeScript logic.

**What works now**
- Extension manifest and Gemini CLI scaffolding are present.
- Core Rust modules exist for agents, hooks, MCP/ACP, sessions, and RPC.
- Command and documentation sets are present (currently being refreshed).

**In progress**
- TypeScript → Rust parity for large subsystems (background agents, config,
  ACP normalization).
- End‑to‑end session flows for Gemini/Codex/Qwen within a unified adapter.
- Validation of hook behavior and task orchestration across agents.

**Known gaps**
- Some Rust modules compile but are not fully wired end‑to‑end.
- Configuration loading/migration is still being aligned to actual runtime.
- Authentication flows for some CLIs still require manual steps.

**What to expect right now**
- You can explore the architecture, commands, and agent catalogs.
- Some workflows will still require manual setup or troubleshooting.

For a complete non‑developer overview, see `docs/PROJECT_STATUS.md`.

---
alwaysApply: false
name: prompt-template-system
description: Agent for prompt template system
category: utility
---

# TASK
[คำสั่งงาน]

User: "{user_input}"
Assistant:
"""
```

3. **อัปเดต templates dictionary:**
```python
self.templates = {
    PromptType.SCENE_ARCHITECT: self._get_scene_architect_prompt(),
    # เพิ่มใหม่ที่นี่
    PromptType.NEW_TEMPLATE: self._get_new_prompt_template(),
}
```

## Context Injection Rules

### การใช้ Context Manager:
```python
from src.core.context_manager import context_manager
from src.core.prompt_templates import prompt_template_manager, PromptType, PromptContext

# สร้าง context
context = context_manager.get_context_for_prompt(user_id="user123")

# สร้าง prompt
prompt = prompt_template_manager.get_prompt(
    PromptType.SCENE_ARCHITECT,
    context=context
)
```

### Context Placeholders ที่ใช้ได้:
- `{STORY_CONTEXT}` - ข้อมูลเรื่องราว "Bound Fate"
- `{PROJECT_STATUS}` - สถานะโปรเจกต์
- `{USER_PREFERENCES}` - ความชอบของผู้ใช้
- `{AVAILABLE_TOOLS}` - เครื่องมือที่ใช้ได้
- `{ERROR_CONTEXT}` - บริบทข้อผิดพลาด

## Model Routing Rules

### การใช้ Model Router:
```python
from src.core.model_router import model_router

# Route request
model_name, task_category, model_tier = await model_router.route_request(
    user_input="ช่วยเขียนฉากต่อสู้",
    user_preferences=user_prefs
)
```

### Task Categories:
- `SIMPLE_QA` - คำถามง่ายๆ
- `TOOL_USE` - ใช้เครื่องมือ
- `COMPLEX_REASONING` - การคิดวิเคราะห์ซับซ้อน
- `CODE_GENERATION` - สร้างโค้ด
- `CREATIVE_WRITING` - เขียนสร้างสรรค์
- `AMBIGUOUS` - ไม่แน่ชัด

### Model Tiers:
- `LOCAL` - โมเดลในเครื่อง (0 cost)
- `FAST_CLOUD` - โมเดลเร็ว (ต้นทุนต่ำ)
- `SMART_CLOUD` - โมเดลฉลาด (ต้นทุนสูง)

## การทดสอบ Prompt Templates

### สร้างไฟล์ทดสอบ:
```python
# test_prompt_templates.py
async def test_scene_architect():
    context = context_manager.get_context_for_prompt()
    prompt = prompt_template_manager.get_prompt(
        PromptType.SCENE_ARCHITECT,
        context=context
    )
    # ทดสอบ prompt
```

### การตรวจสอบ Context Injection:
- ตรวจสอบว่า placeholders ถูกแทนที่
- ตรวจสอบ JSON format
- ตรวจสอบ encoding (ensure_ascii=False)

## การอัปเดต Context

### อัปเดต Story Context:
```python
context_manager.update_story_context(
    context_type="characters",
    context_data={
        "new_character": {
            "name": "Erik",
            "role": "Mercenary",
            "personality": "Pragmatic"
        }
    }
)
```

### อัปเดต Project Status:
```python
context_manager.update_project_status(
    component="prompt_templates",
    status="active",
    details={
        "templates_available": 7,
        "context_injection": "working"
    }
)
```

## Best Practices

### การเขียน Prompt Template:
1. **ชัดเจนและเฉพาะเจาะจง**
2. **ใช้ placeholders สำหรับ context**
3. **ระบุ output format ที่ต้องการ**
4. **ให้ตัวอย่างการใช้งาน**

### การจัดการ Context:
1. **อัปเดต context ทันทีเมื่อมีการเปลี่ยนแปลง**
2. **ใช้ JSON format สำหรับ structured data**
3. **บันทึกใน database สำหรับ persistence**
4. **ใช้ in-memory cache สำหรับ performance**

### การเลือก Model:
1. **วิเคราะห์ task complexity ก่อน**
2. **พิจารณา cost optimization**
3. **เคารพ user preferences**
4. **มี fallback mechanism**
description:
globs:
alwaysApply: true
---
