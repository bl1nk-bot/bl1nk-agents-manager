# Feature Development Rules

## Overview

This document defines the workflow for creating new features in the BL1NK Agents Manager project. Following these rules ensures consistency, proper schema management, and maintainable codebase.

---

## Feature Creation Workflow

### Step 1: Identify Feature Category

Determine if the feature is:

1. **Configuration Feature** - Settings, TOML configs, feature flags
2. **Schema-driven Feature** - Data structures with JSON Schema
3. **Interface Feature** - Internal or external API boundaries
4. **Protocol Feature** - MCP/ACP communication

### Step 2: Schema Check (Schema-driven Features)

**Before starting any schema-driven feature:**

1. Check if a schema file already exists for this feature group:
   ```
   .config/
   ├── schema-agent.json       # Agent specification schema
   ├── schema-task.json       # Task schema (if exists)
   └── schema-*.json          # Other feature schemas
   ```

2. If schema does NOT exist in the group:
   - ✅ **Create JSON Schema Draft 07 first**
   - Define all fields, types, constraints
   - Document enum values and defaults
   - Only then proceed to Step 3

3. If schema EXISTS:
   - ✅ **Review and validate the schema**
   - Ensure new fields are backward compatible
   - Update schema version if needed
   - Lock the schema (commit with `[schema-lock]` prefix)
   - Only then proceed to Step 3

### Step 3: Schema Lock Before Implementation

**Schema must be locked before code implementation:**

```
┌─────────────────────────────────────────┐
│  Create/Review Schema (Draft 07)        │
│  └── Document all fields & constraints   │
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│  Schema Review & Lock                  │
│  └── Commit with [schema-lock] prefix  │
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│  Implement Feature                      │
│  └── Use locked schema as source of truth
└─────────────────────────────────────────┘
```

### Step 4: Interface Features (Internal/External)

When creating features that may change interfaces:

**Internal Interfaces:**
- Document in source code with doc comments
- Update module-level documentation
- Add to ARCHITECTURE.md if architectural impact

**External Interfaces:**
- ✅ Must have JSON Schema
- ✅ Must be versioned
- ✅ Must have changelog entry

---

## Schema Management

### JSON Schema Draft 07 Template

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "$id": "https://github.com/billlzzz18/bl1nk-agents-manager/schema/<feature>.json",
  "title": "<Feature> Schema",
  "description": "Description of the feature schema",
  "version": "1.0.0",
  "type": "object",
  "properties": {
    "field_name": {
      "type": "string",
      "description": "Field description"
    }
  },
  "required": ["field_name"],
  "additionalProperties": false
}
```

### Schema File Location

| Feature Type | Location |
|-------------|----------|
| Core schemas | `.config/schema-*.json` |
| Agent schemas | `.config/schema-agent.json` |
| Protocol schemas | `.config/schema-mcp.json` |
| Hook schemas | `.config/schema-hook.json` |

---

## Feature Checklist

Before starting implementation, verify:

- [ ] Feature category identified
- [ ] Schema exists or created (Draft 07)
- [ ] Schema reviewed and locked
- [ ] Version bumped if schema updated
- [ ] Documentation updated
- [ ] Tests planned

---

## Commit Message Convention

### Schema Changes

```
[schema-lock] <feature>: lock schema v1.0.0
[schema] <feature>: add/update schema fields
```

### Feature Changes

```
feat(<feature>): add new feature
fix(<feature>): fix bug
refactor(<feature>): restructure
docs(<feature>): update docs
```

---

## Related Files

- `.config/schema-agent.json` - Agent specification schema
- `src/config.rs` - Configuration management
- `src/agents/creator.rs` - Agent creation logic
- `docs/ARCHITECTURE.md` - System architecture

---

*Last updated: 2026-04-11*
