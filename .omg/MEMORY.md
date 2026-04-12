# Project Memory — BL1NK Agents Manager

> High-signal index. Details in `.omg/memory/` topic files.

---

## Architecture
- **Product:** Intelligent MCP/ACP orchestrator with bundled PMAT support
- **Core:** Rust 2024, tokio async, clap CLI
- **Protocol:** MCP (pmcp 1.8) + ACP (agent-client-protocol 0.10)
- **Kilo Integration:** GitHub Actions workflow with qwen/qwen3.6-plus:free via Kilo Gateway
- **Conductor:** Spec-driven development workflow enabled

## Key Decisions
| Date | Decision | Reason |
|------|----------|--------|
| 2026-04-12 | KiloCode shell ไม่ใช่ Qwen Code | User correction — agent วาง skill ผิดที่ |
| 2026-04-12 | ใช้แค่ KILO_API_KEY | ไม่ต้องใช้ ANTHROPIC/OPENAI keys |
| 2026-04-12 | Kilo skills ใน `.kilo/skills/` | ไม่ใช่ `.qwen/skills/` |
| 2026-04-12 | ลบ manual commit step จาก Kilo workflow | OIDC จัดการ git credentials เอง |
| 2026-04-12 | Rust `DateTime<Utc>` → `String` สำหรับ JsonSchema | schemars 1.0 ไม่มี impl สำหรับ DateTime |

## Critical Patterns
- Git corruption บน Android/Termux → `rm corrupted object` + `git reset --hard`
- Parallel CI checks → fmt+clippy+test พร้อมกัน (ประหยัด 2-3x)
- Output language: Thai เสมอ
- อย่ากลัวความผิดพลาด ใช้ `ask_user` เมื่อไม่แน่ใจ
- สร้าง scripts สำหรับงานซ้ำๆ

## Active Track
- **Track:** Unified Registry, Monitoring & Honesty Layer
- **Phase:** 1 - Foundation (Task 1.1 complete, Task 1.2 next)
- **Commit:** `afead8b`

## See Also
- [Conductor Product](../../conductor/product.md)
- [Conductor Tech Stack](../../conductor/tech-stack.md)
- [Conductor Workflow](../../conductor/workflow.md)
- [Learned Rules](./rules/learned/)
- [Memory Topics](./memory/)
