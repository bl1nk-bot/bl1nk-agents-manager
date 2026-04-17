---
name: kilo-secrets-minimal
description: >
  Kilo workflow ใช้แค่ KILO_API_KEY เท่านั้น ไม่ต้องใช้ API keys อื่น
globs:
  - ".github/workflows/kilo.yml"
---

# Kilo Secrets - ใช้แค่ KILO_API_KEY

## Required Secrets

| Secret | จำเป็น | หมายเหตุ |
|--------|--------|----------|
| `KILO_API_KEY` | ✅ ต้องใส่ | ได้จาก kilo.ai dashboard |
| `KILO_ORG_ID` | ❌ ไม่บังคับ | ใช้ถ้ามี org เท่านั้น |

## ไม่ต้องใช้

- ❌ `ANTHROPIC_API_KEY` - Kilo จัดการให้ผ่าน gateway
- ❌ `OPENAI_API_KEY` - Kilo จัดการให้ผ่าน gateway
- ❌ `GITHUB_TOKEN` - GitHub Actions สร้างให้อัตโนมัติ
- ❌ GitHub PAT - ใช้ `use_github_token: true` ได้เลย

## Kilo Gateway

```yaml
oidc_base_url: "https://api.kilo.ai/api/gateway"
```
