# List Hookify Rules

**Load hookify:writing-rules skill first** to understand rule format.

Show all configured hookify rules in the project.

## Steps

1. Use Glob tool to find all hookify rule files:

   ```toml
   pattern: ".claude/hookify.*.local.md"
   ```

2. For each file found:
   - Use Read tool to read the file
   - Extract frontmatter fields: name, enabled, event, pattern
   - Extract message preview (first 100 chars)

3. Present results in a table:

```markdown
## Configured Hookify Rules

| Name | Enabled | Event | Pattern | File |
|------|---------|-------|---------|------|
| warn-dangerous-rm | ✅ Yes | bash | rm\s+-rf | hookify.dangerous-rm.local.md |
| warn-console-log | ✅ Yes | file | console\.log\( | hookify.console-log.local.md |
| check-tests | ❌ No | stop | .* | hookify.require-tests.local.md |

**Total**: 3 rules (2 enabled, 1 disabled)
```

1. For each rule, show a brief preview:

```markdown
### warn-dangerous-rm
**Event**: bash
**Pattern**: `rm\s+-rf`
**Message**: "⚠️ **Dangerous rm command detected!** This command could delete..."

**Status**: ✅ Active
**File**: .claude/hookify.dangerous-rm.local.md

---

To modify a rule: Edit the .local.md file directly
To disable a rule: Set `enabled: false` in frontmatter
To enable a rule: Set `enabled: true` in frontmatter
To delete a rule: Remove the .local.md file
To create a rule: Use `/hookify` command

**Remember**: Changes take effect immediately - no restart needed

If no hookify rules exist:

```markdown
## No Hookify Rules Configured

You haven't created any hookify rules yet.

To get started:
1. Use `/hookify` to analyze conversation and create rules
2. Or manually create `.claude/hookify.my-rule.local.md` files
3. See `/hookify:help` for documentation
```

Example:

```markdown
/hookify Warn me when I use console.log
```
