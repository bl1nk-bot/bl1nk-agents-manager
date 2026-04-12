# Safety Plugin (P0)

Block dangerous operations before they execute.

**Fail Policy:** Fail-closed | **Timeout:** 400ms | **Debounce:** 0ms

## Triggers
- Exposed secrets/credentials → BLOCK
- Destructive file operations without confirmation → BLOCK
- Unsafe shell commands (rm -rf, etc.) → BLOCK
- Permission escalation attempts → BLOCK
- Hardcoded API keys in diff → BLOCK

## Rules
- kilo-secrets-minimal
- output-language-thai (alwaysApply)
