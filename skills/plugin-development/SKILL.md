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
name: plugin-development
description: Expert Claude Code plugin development covering plugin structure, slash
  commands, auto-activating skills, sub-agents, plugin.json configuration, YAML frontmatter,
  activation keywords, directory structure, and plugin best practices. Activates for
  plugin development, create plugin, claude plugin, slash command, skill activation,
  SKILL.md, plugin.json, claude code plugin, how to make plugin.
metadata:
  version: 1.0.0
  quality: FAIR
  last_updated: 2025-12-28 07:28:22.133482
---



























# Plugin Development Expert

Expert guidance for creating production-ready Claude Code plugins.

## Critical Structure Rules

**Directory Hierarchy**:
```
~/.claude/plugins/my-plugin/    ← Plugin root
├── .claude-plugin/
│   └── plugin.json            ← Manifest (REQUIRED)
├── commands/
│   └── command-name.md        ← Slash commands
├── skills/
│   └── skill-name/            ← MUST be subdirectory
│       └── SKILL.md           ← MUST be uppercase
└── agents/
    └── agent-name/
        └── AGENT.md
```

**Common Mistakes**:
```
# ❌ WRONG
skills/SKILL.md                # Missing subdirectory
skills/my-skill.md             # Wrong filename
skills/My-Skill/SKILL.md       # CamelCase not allowed

# ✅ CORRECT
skills/my-skill/SKILL.md       # kebab-case subdirectory + SKILL.md
```

## plugin.json Format

**Minimum Required**:
```json
{
  "name": "my-plugin",
  "description": "Clear description with activation keywords",
  "version": "1.0.0"
}
```

**Full Example**:
```json
{
  "name": "my-awesome-plugin",
  "description": "Expert cost optimization for AWS, Azure, GCP. Activates for reduce costs, cloud costs, finops, save money, cost analysis.",
  "version": "1.0.0",
  "author": {
    "name": "Your Name",
    "email": "you@example.com"
  },
  "homepage": "https://github.com/user/my-plugin",
  "repository": "https://github.com/user/my-plugin",
  "license": "MIT",
  "keywords": ["cost", "finops", "aws", "azure", "gcp" ]
}
```

## Command Format (Slash Commands)

**Header Format** (CRITICAL):
```markdown
# /my-plugin:command-name
```

**Rules**:
- MUST start with `# /`
- Plugin name: `kebab-case`
- Command name: `kebab-case`
- NO YAML frontmatter (only skills use YAML)

**Full Template**:
```markdown
# /my-plugin:analyze-costs

Analyze cloud costs and provide optimization recommendations.

You are an expert FinOps engineer.

## Your Task

1. Collect cost data
2. Analyze usage patterns
3. Identify optimization opportunities
4. Generate report

### 1. Data Collection

\```bash
aws ce get-cost-and-usage --time-period...
\```

## Example Usage

**User**: "Analyze our AWS costs"

**Response**:
- Pulls Cost Explorer data
- Identifies $5K/month in savings
- Provides implementation plan

## When to Use

- Monthly cost reviews
- Budget overruns
- Pre-purchase planning
```

## Skill Format (Auto-Activating)

**YAML Frontmatter** (REQUIRED):
```yaml
---
name: cost-optimization
description: Expert cloud cost optimization for AWS, Azure, GCP. Covers FinOps, reserved instances, spot instances, right-sizing, storage optimization. Activates for reduce costs, save money, cloud costs, aws costs, finops, cost optimization, budget overrun, expensive bill.
---
```

**Activation Keywords**:
```yaml
# ✅ GOOD: Specific, varied keywords
description: Expert Python optimization. Activates for python performance, optimize python code, speed up python, profiling, cProfile, pypy, numba.

# ❌ BAD: Too generic
description: Python expert.

# ❌ BAD: No activation keywords
description: Expert Python optimization covering performance tuning.
```

**Full Template**:
```markdown
---
name: my-skill
description: Expert [domain ] covering [topics ]. Activates for keyword1, keyword2, phrase3, action4.
---

# Skill Title

You are an expert [role ] with deep knowledge of [domain ].

## Core Expertise

### 1. Topic Area

Content here...

### 2. Code Examples

\```typescript
// Working examples
\```

## Best Practices

- Practice 1
- Practice 2

You are ready to help with [domain ]!
```

## Agent Format (Sub-Agents)

**File Location**:
```
agents/agent-name/AGENT.md
```

**Template**:
```markdown
---
name: specialist-agent
description: Specialized agent for [specific task ]
---

# Agent Title

You are a specialized agent for [purpose ].

## Capabilities

1. Capability 1
2. Capability 2

## Workflow

1. Analyze input
2. Execute specialized task
3. Return results
```

**Invocation**:
```typescript
Task({
  subagent_type: "plugin-name:folder-name:yaml-name",
  prompt: "Task description"
});

// Example
Task({
  subagent_type: "my-plugin:specialist-agent:specialist-agent",
  prompt: "Analyze this code for security vulnerabilities"
});
```

## Testing Workflow

**1. Install Plugin**:
```bash
cp -r my-plugin ~/.claude/plugins/
# OR
claude plugin add github:username/my-plugin
```

**2. Restart Claude Code**:
```bash
# Required after:
- Adding new plugin
- Modifying plugin.json
- Adding/removing commands
- Changing YAML frontmatter
```

**3. Test Commands**:
```bash
# Type "/" in Claude Code
# Verify command appears: /my-plugin:command-name
# Execute command
# Verify behavior
```

**4. Test Skills**:
```bash
# Ask trigger question: "How do I reduce costs?"
# Verify skill activates
# Check response uses skill knowledge
```

**5. Check Logs**:
```bash
tail -f ~/.claude/logs/claude.log | grep my-plugin

# Expected:
# ✅ "Loaded plugin: my-plugin"
# ✅ "Registered command: /my-plugin:analyze"
# ✅ "Registered skill: cost-optimization"

# Errors:
# ❌ "Failed to parse plugin.json"
# ❌ "YAML parsing error in SKILL.md"
# ❌ "Command header malformed"
```

## Common Issues

**Issue: Skill not activating**
```
Checklist:
1. ✅ YAML frontmatter present? (---...---)
2. ✅ Activation keywords in description?
3. ✅ SKILL.md in subdirectory? (skills/name/SKILL.md)
4. ✅ File named SKILL.md (uppercase)?
5. ✅ Claude Code restarted?
```

**Issue: Command not found**
```
Checklist:
1. ✅ Header format: # /plugin-name:command-name
2. ✅ File in commands/ directory?
3. ✅ Plugin name matches plugin.json?
4. ✅ Claude Code restarted?
```

**Issue: YAML parsing error**
```
Common causes:
- Unclosed quotes: description: "Missing end
- Invalid characters: name: my_skill (use hyphens)
- Missing closing ---
- Incorrect indentation
```

## Best Practices

**Naming**:
- Plugin: `my-awesome-plugin` (kebab-case)
- Commands: `analyze-costs` (kebab-case)
- Skills: `cost-optimization` (kebab-case)
- NO underscores, NO CamelCase

**Activation Keywords**:
- Include 5-10 trigger keywords
- Mix specific terms and common phrases
- Think about what users will ask
- Test with real questions

**Documentation**:
- Clear "Your Task" section
- Code examples with syntax highlighting
- "Example Usage" section
- "When to Use" section

**Performance**:
- Keep SKILL.md under 50KB
- Optimize command prompts
- Avoid expensive operations

Create production-ready Claude Code plugins!
