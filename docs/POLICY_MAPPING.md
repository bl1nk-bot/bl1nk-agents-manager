# 🛡️ Policy Engine Mapping: bl1nk-agents vs. Gemini CLI

เพื่อให้ระบบ Permission ของเรามีความเป็นสากลและปลอดภัยสูงสุด เราจะปรับปรุงโครงสร้างตามมาตรฐานของ Gemini CLI Policy Engine ดังนี้:

## 1. Permission Hierarchy (5-Tier)

เราจะแมป `permission` score (0-999) เข้ากับระบบ Tier ของ Gemini:

- **Admin (900-999)**: เอเจนต์ที่ได้รับความไว้วางใจสูงสุด (เช่น Orchestrator)
- **User/Workspace (700-899)**: เอเจนต์ที่สร้างขึ้นเพื่อใช้งานเฉพาะโครงการ
- **Default/Extension (100-699)**: เอเจนต์ทั่วไปหรือ Skill Knowledge

## 2. Decision Logic

แทนที่จะใช้แค่คะแนนดิบ เราจะเพิ่ม `PolicyEvaluator` ที่ตรวจสอบ:

- **`match_rules`**: ตรวจสอบ `toolName` และ `args` ผ่าน Regex
- **`mode_context`**: สิทธิ์จะเปลี่ยนไปตามโหมด (เช่น ในโหมด `plan` ทุกอย่างจะเป็น `allow` สำหรับงานอ่านไฟล์ แต่ในโหมด `execute` จะเป็น `ask_user`)

## 3. Implementation Plan

1. อัปเดต `schema-agent.json` ให้รองรับโครงสร้างกฎระเบียบ
2. เพิ่ม `PolicyEvaluator` struct ในโมดูล Registry
3. เชื่อมต่อ `AgentExecutor` ให้เรียกใช้ Evaluator ก่อนรัน `bash` หรือ `write`
