---
name: plugin-development
description: This skill should be used when creating plugins, writing skills, building commands, developing agents, or asking about "plugin development", "create skill", "write command", "build agent", "SKILL.md", "plugin structure", "progressive disclosure".
version: 1.2.1
---

# Plugin Development Guide

General guide for creating effective plugins with skills, commands, and agents.

## Plugin Structure

```text
my-plugin/
├── .metadata/
│   └── manifest.json       # Plugin metadata
├── skills/
│   └── skill-name/
│       ├── SKILL.md        # Required
│       ├── references/     # Detailed docs (Optional)
│       └── scripts/        # Utilities (Optional)
├── commands/
│   └── command-name.md
├── agents/
│   └── agent-name.md
└── hooks/
    └── hooks.json
```

## Skill Development

### SKILL.md Structure

```yaml
---
name: skill-name
description: This skill should be used when the user asks to "specific phrase 1", "specific phrase 2", or mentions "keyword". Be specific about triggers.
version: 1.0.0
---

# Skill Title

Core content here.

## Additional Resources (Optional)

- **`references/detailed.md`** - Detailed patterns
- **`examples/`** - Working examples (Include only if requested or necessary)
```

### Progressive Disclosure

| Level | Content | When Loaded |
|-------|---------|-------------|
| **Metadata** | name + description | Always (~100 words) |
| **SKILL.md** | Core content | When triggered (<5k words) |
| **References** | Detailed docs | As needed (Optional) |
| **Examples** | Working code | **Optional** (Based on user request) |

### Description Best Practices

**Good:**
```yaml
description: This skill should be used when the user asks to "create a hook", "add PreToolUse hook", "validate tool use", or mentions hook events.
```

**Bad:**
```yaml
description: Provides hook guidance.  # Too vague
description: Use this skill for hooks.  # Not third person
```

### Writing Style

Use **imperative form**, not second person:

```markdown
# Good
Start by reading the configuration.
Validate the input before processing.

# Bad
You should start by reading...
You need to validate the input...
```

## Command Development

### Command Structure

```yaml
---
name: command-name
description: What the command does
argument-hint: "[optional args]"
---

# Command Title

Instructions for executing the command.
```

## Agent Development

### Agent Structure

```yaml
---
agent: agent-name
description: |
  When to use this agent with examples:
  <example>
  Context: User situation
  user: "User request"
  assistant: "How assistant responds"
  <commentary>Why this agent is appropriate</commentary>
  </example>
tools:
  - Read
  - Glob
  - Grep
  - Bash
color: cyan
---

# Agent Instructions

Detailed instructions for the agent's behavior.
```

## Hooks Development

### hooks.json Structure

```json
{
  "hooks": [
    {
      "event": "PreToolUse",
      "matcher": "Write|Edit",
      "type": "prompt",
      "prompt": "Validate code before writing...",
      "timeout": 10000
    }
  ]
}
```

## Validation Checklist

**Skills:**
- [ ] SKILL.md has valid YAML frontmatter
- [ ] Description uses third person with trigger phrases
- [ ] Body is detailed (use references/ if content is very large)
- [ ] Uses imperative writing style
- [ ] Referenced files exist (if any)

**Commands:**
- [ ] Has name and description in frontmatter
- [ ] Clear instructions for execution
- [ ] argument-hint if accepts parameters

**Agents:**
- [ ] Has description with examples
- [ ] Specifies required tools
- [ ] Valid color specified
- [ ] Detailed behavioral instructions

**Optional Elements:**
- [ ] Examples are provided only if explicitly requested or essential for clarity
- [ ] References are used to keep the main SKILL.md concise

## Common Mistakes
- **Weak skill descriptions** - Be specific with trigger phrases
- **Too much in SKILL.md** - Use progressive disclosure
- **Second person writing** - Use imperative form
- **Missing resource references** - Point to references/ as needed
- **Vague agent examples** - Include concrete user/assistant pairs
