# Migrating from Basic to Advanced Hooks
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

This guide shows how to migrate from basic command hooks to advanced prompt-based hooks for better maintainability and flexibility.

## Why Migrate?

Prompt-based hooks offer several advantages:

- **Natural language reasoning**: LLM understands context and intent
- **Better edge case handling**: Adapts to unexpected scenarios
- **No bash scripting required**: Simpler to write and maintain
- **More flexible validation**: Can handle complex logic without coding

## Migration Example: Bash Command Validation

### Before (Basic Command Hook)

**Configuration:**
```json
{
  "PreToolUse": [
    {
      "matcher": "Bash",
      "hooks": [
        {
          "type": "command",
          "command": "bash validate-bash.sh"
        }
      ]
    }
  ]
}
```

**Script (validate-bash.sh):**
```bash
#!/bin/bash
input=$(cat)
command=$(echo "$input" | jq -r '.tool_input.command')

# Hard-coded validation logic
if [[ "$command" == *"rm -rf"* ]]; then
  echo "Dangerous command detected" >&2
  exit 2
fi
```

**Problems:**
- Only checks for exact "rm -rf" pattern
- Doesn't catch variations like `rm -fr` or `rm -r -f`
- Misses other dangerous commands (`dd`, `mkfs`, etc.)
- No context awareness
- Requires bash scripting knowledge

### After (Advanced Prompt Hook)

**Configuration:**
```json
{
  "PreToolUse": [
    {
      "matcher": "Bash",
      "hooks": [
        {
          "type": "prompt",
          "prompt": "Command: $TOOL_INPUT.command. Analyze for: 1) Destructive operations (rm -rf, dd, mkfs, etc) 2) Privilege escalation (sudo) 3) Network operations without user consent. Return 'approve' or 'deny' with explanation.",
          "timeout": 15
        }
      ]
    }
  ]
}
```

**Benefits:**
- Catches all variations and patterns
- Understands intent, not just literal strings
- No script file needed
- Easy to extend with new criteria
- Context-aware decisions
- Natural language explanation in denial

## Migration Example: File Write Validation

### Before (Basic Command Hook)

**Configuration:**
```json
{
  "PreToolUse": [
    {
      "matcher": "Write",
      "hooks": [
        {
          "type": "command",
          "command": "bash validate-write.sh"
        }
      ]
    }
  ]
}
```

**Script (validate-write.sh):**
```bash
#!/bin/bash
input=$(cat)
file_path=$(echo "$input" | jq -r '.tool_input.file_path')

# Check for path traversal
if [[ "$file_path" == *".."* ]]; then
  echo '{"decision": "deny", "reason": "Path traversal detected"}' >&2
  exit 2
fi

# Check for system paths
if [[ "$file_path" == "/etc/"* ]] || [[ "$file_path" == "/sys/"* ]]; then
  echo '{"decision": "deny", "reason": "System file"}' >&2
  exit 2
fi
```

**Problems:**
- Hard-coded path patterns
- Doesn't understand symlinks
- Missing edge cases (e.g., `/etc` vs `/etc/`)
- No consideration of file content

### After (Advanced Prompt Hook)

**Configuration:**
```json
{
  "PreToolUse": [
    {
      "matcher": "Write|Edit",
      "hooks": [
        {
          "type": "prompt",
          "prompt": "File path: $TOOL_INPUT.file_path. Content preview: $TOOL_INPUT.content (first 200 chars). Verify: 1) Not system directories (/etc, /sys, /usr) 2) Not credentials (.env, tokens, secrets) 3) No path traversal 4) Content doesn't expose secrets. Return 'approve' or 'deny'."
        }
      ]
    }
  ]
}
```

**Benefits:**
- Context-aware (considers content too)
- Handles symlinks and edge cases
- Natural understanding of "system directories"
- Can detect secrets in content
- Easy to extend criteria

## When to Keep Command Hooks

Command hooks still have their place:

### 1. Deterministic Performance Checks

```bash
#!/bin/bash
# Check file size quickly
file_path=$(echo "$input" | jq -r '.tool_input.file_path')
size=$(stat -f%z "$file_path" 2>/dev/null || stat -c%s "$file_path" 2>/dev/null)

if [ "$size" -gt 10000000 ]; then
  echo '{"decision": "deny", "reason": "File too large"}' >&2
  exit 2
fi
```

**Use command hooks when:** Validation is purely mathematical or deterministic.

### 2. External Tool Integration

```bash
#!/bin/bash
# Run security scanner
file_path=$(echo "$input" | jq -r '.tool_input.file_path')
scan_result=$(security-scanner "$file_path")

if [ "$?" -ne 0 ]; then
  echo "Security scan failed: $scan_result" >&2
  exit 2
fi
```

**Use command hooks when:** Integrating with external tools that provide yes/no answers.

### 3. Very Fast Checks (< 50ms)

```bash
#!/bin/bash
# Quick regex check
command=$(echo "$input" | jq -r '.tool_input.command')

if [[ "$command" =~ ^(ls|pwd|echo)$ ]]; then
  exit 0  # Safe commands
fi
```

**Use command hooks when:** Performance is critical and logic is simple.

## Hybrid Approach

Combine both for multi-stage validation:

```json
{
  "PreToolUse": [
    {
      "matcher": "Bash",
      "hooks": [
        {
          "type": "command",
          "command": "bash ${CLAUDE_PLUGIN_ROOT}/scripts/quick-check.sh",
          "timeout": 5
        },
        {
          "type": "prompt",
          "prompt": "Deep analysis of bash command: $TOOL_INPUT",
          "timeout": 15
        }
      ]
    }
  ]
}
```

The command hook does fast deterministic checks, while the prompt hook handles complex reasoning.

## Migration Checklist

When migrating hooks:

- [ ] Identify the validation logic in the command hook
- [ ] Convert hard-coded patterns to natural language criteria
- [ ] Test with edge cases the old hook missed
- [ ] Verify LLM understands the intent
- [ ] Set appropriate timeout (usually 15-30s for prompt hooks)
- [ ] Document the new hook in README
- [ ] Remove or archive old script files

## Migration Tips

1. **Start with one hook**: Don't migrate everything at once
2. **Test thoroughly**: Verify prompt hook catches what command hook caught
3. **Look for improvements**: Use migration as opportunity to enhance validation
4. **Keep scripts for reference**: Archive old scripts in case you need to reference the logic
5. **Document reasoning**: Explain why prompt hook is better in README

## Complete Migration Example

### Original Plugin Structure

```
my-plugin/
├── .claude-plugin/plugin.json
├── hooks/hooks.json
└── scripts/
    ├── validate-bash.sh
    ├── validate-write.sh
    └── check-tests.sh
```

### After Migration

```
my-plugin/
├── .claude-plugin/plugin.json
├── hooks/hooks.json      # Now uses prompt hooks
└── scripts/              # Archive or delete
    └── archive/
        ├── validate-bash.sh
        ├── validate-write.sh
        └── check-tests.sh
```

### Updated hooks.json

```json
{
  "PreToolUse": [
    {
      "matcher": "Bash",
      "hooks": [
        {
          "type": "prompt",
          "prompt": "Validate bash command safety: destructive ops, privilege escalation, network access"
        }
      ]
    },
    {
      "matcher": "Write|Edit",
      "hooks": [
        {
          "type": "prompt",
          "prompt": "Validate file write safety: system paths, credentials, path traversal, content secrets"
        }
      ]
    }
  ],
  "Stop": [
    {
      "matcher": "*",
      "hooks": [
        {
          "type": "prompt",
          "prompt": "Verify tests were run if code was modified"
        }
      ]
    }
  ]
}
```

**Result:** Simpler, more maintainable, more powerful.

## Common Migration Patterns

### Pattern: String Contains → Natural Language

**Before:**
```bash
if [[ "$command" == *"sudo"* ]]; then
  echo "Privilege escalation" >&2
  exit 2
fi
```

**After:**
```
"Check for privilege escalation (sudo, su, etc)"
```

### Pattern: Regex → Intent

**Before:**
```bash
if [[ "$file" =~ \.(env|secret|key|token)$ ]]; then
  echo "Credential file" >&2
  exit 2
fi
```

**After:**
```
"Verify not writing to credential files (.env, secrets, keys, tokens)"
```

### Pattern: Multiple Conditions → Criteria List

**Before:**
```bash
if [ condition1 ] || [ condition2 ] || [ condition3 ]; then
  echo "Invalid" >&2
  exit 2
fi
```

**After:**
```
"Check: 1) condition1 2) condition2 3) condition3. Deny if any fail."
```

## Conclusion

Migrating to prompt-based hooks makes plugins more maintainable, flexible, and powerful. Reserve command hooks for deterministic checks and external tool integration.
