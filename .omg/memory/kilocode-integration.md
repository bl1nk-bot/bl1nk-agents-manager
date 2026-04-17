# KiloCode Integration

## Overview

KiloCode เป็น shell หลัก ใช้ Qwen model (qwen/qwen3.6-plus:free) ผ่าน Kilo Gateway

## Configuration

```yaml
model: qwen/qwen3.6-plus:free
oidc_base_url: "https://api.kilo.ai/api/gateway"
```

## Secrets

| Secret | จำเป็น | หมายเหตุ |
|--------|--------|----------|
| `KILO_API_KEY` | ✅ | ได้จาก kilo.ai dashboard |
| `KILO_ORG_ID` | ❌ | ใช้ถ้ามี org เท่านั้น |

## ไม่ต้องใช้

- ❌ ANTHROPIC_API_KEY
- ❌ OPENAI_API_KEY
- ❌ GitHub PAT

## Skills Location

- วางใน `.kilo/skills/` ไม่ใช่ `.qwen/skills/`
- Review skill: `.kilo/skills/review.md`

## GitHub Workflows

- `.github/workflows/kilo.yml` — Kilo automation on /kilo or /kc comments
- `.github/workflows/ci.yml` — CI pipeline with parallel checks
- `.github/workflows/security.yml` — Security audit workflow

## Lessons Learned

- ครั้งแรกวาง skill ผิดที่ (ใน .qwen/skills/) → ผู้ใช้แก้ไข → ย้ายไป .kilo/skills/
- อย่ามี manual commit step ใน workflow ที่ใช้ `persist-credentials: false`
- Kilo จัดการ git credentials ผ่าน OIDC เอง
