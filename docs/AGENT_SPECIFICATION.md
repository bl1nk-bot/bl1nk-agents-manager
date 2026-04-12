# Agent Specification for Bl1nk Agents Manager

**Version:** 1.0  
**Date:** 2026-02-07  
**Status:** FINAL - DO NOT MODIFY WITHOUT APPROVAL

---

## Purpose

This document defines the **EXACT specification** for agent files in the Bl1nk Agents Manager project. All agents MUST conform to this specification.

This spec is designed to work with **BOTH**:
- Claude Code
- Gemini CLI

**Do NOT add platform-specific fields** (model, temperature, tools, kind, etc.) to agent files.

---

## File Format

### File Naming
- Extension: `.md`
- Naming: `{agent-name}.md`
- Format: lowercase with hyphens (e.g., `code-architect.md`)
- Location: `agents/` directory

### File Structure

```markdown
---
name: {agent-name}
description: {description-text}
category: {category}          # optional
color: {color}                # optional
tools: ["Tool1", "Tool2"]     # optional
model: inherit                # optional
---

{system-prompt}
```

**Required fields:** `name`, `description`  
**Optional fields:** `category`, `color`, `tools`, `model`

---

## Field Specifications

### Required Fields

#### 1. `name`
- **Type:** string
- **Format:** lowercase with hyphens only
- **Length:** 3-50 characters
- **Pattern:** `^[a-z0-9]+(-[a-z0-9]+)*$`
- **Examples:** 
  - ✅ `code-architect`
  - ✅ `security-auditor`
  - ✅ `test-generator`
  - ❌ `Code-Architect` (no capitals)
  - ❌ `code_architect` (no underscores)
  - ❌ `ca` (too short)

#### 2. `description`
- **Type:** string
- **Format:** Plain text description of agent purpose
- **Length:** 50-500 characters
- **Guidelines:**
  - Clear and concise
  - Describe what the agent does
  - No trigger examples (deprecated)
  - No XML tags
  - Focus on functionality

**Example:**
```yaml
description: Designs feature architectures by analyzing existing codebase patterns and conventions, then providing comprehensive implementation blueprints with specific files to create/modify, component designs, data flows, and build sequences.
```

---

## Optional Fields

These fields are **OPTIONAL** but recommended:

### 3. `color`
- **Type:** string
- **Purpose:** Visual categorization in UI
- **Valid values:** `blue`, `cyan`, `green`, `yellow`, `red`, `magenta`
- **Semantic meaning:**
  - `blue`/`cyan`: Analysis, review, inspection
  - `green`: Generation, creation, building
  - `yellow`: Validation, caution, warnings
  - `red`: Security, critical operations
  - `magenta`: Transformation, creative tasks

**Example:**
```yaml
color: magenta
```

### 4. `category`
- **Type:** string
- **Purpose:** Logical grouping of agents
- **Format:** lowercase single word or hyphenated
- **Examples:** `utility`, `engineering`, `security`, `analysis`

**Example:**
```yaml
category: utility
```

### 5. `tools`
- **Type:** array of strings
- **Purpose:** Restrict agent to specific tools (reduces context window)
- **Format:** JSON array format
- **When to use:**
  - Specify minimal tools needed for focused agents
  - Omit for agents needing full tool access
- **Benefits:**
  - Reduces context window size
  - More precise agent behavior
  - Faster execution

**Example:**
```yaml
tools: ["Read", "Write"]
```

**Valid tool names:**
- `Read`, `Write`, `Bash`
- `Glob`, `Grep`, `LS`
- `NotebookRead`, `WebFetch`, `WebSearch`
- `TodoWrite`, `KillShell`, `BashOutput`

### 6. `model`
- **Type:** string
- **Purpose:** Specify model for agent
- **Default:** `inherit` (use session model)
- **Options:** `inherit`, `sonnet`, `haiku`, `opus`
- **When to specify:**
  - `sonnet`: Complex reasoning, architecture, analysis
  - `haiku`: Simple, fast tasks
  - `opus`: Maximum capability (rare)

**Example:**
```yaml
model: inherit
```

## Forbidden Fields

These fields should NOT be included (platform-specific runtime config):

❌ `temperature` - Configure at runtime, not in agent file  
❌ `max_turns` - Configure at runtime, not in agent file  
❌ `kind` - Gemini-specific, not needed in universal format

---

## System Prompt Structure

After the frontmatter, include the system prompt with clear sections.

### Recommended Sections

```markdown
# {Agent Title}

{Brief role description}

## Core Responsibilities
1. {Responsibility 1}
2. {Responsibility 2}
3. {Responsibility 3}

## Process
### 1. {Step Name}
{Detailed instructions}

### 2. {Next Step}
{More instructions}

## Output Format
{Expected output format}

## Quality Standards
- {Standard 1}
- {Standard 2}

## Edge Cases
### {Case 1}
{How to handle}

### {Case 2}
{How to handle}
```

### System Prompt Guidelines

**Length:** 500-3,000 words
**Style:** Clear, direct, actionable
**Structure:** Use markdown headers
**Detail:** Specific methodologies, not vague instructions

---

## Complete Example

```markdown
---
name: security-auditor
description: Analyzes code for security vulnerabilities including SQL injection, XSS, hardcoded credentials, and unsafe file operations. Provides clear explanations and suggested fixes without modifying code.
category: security
color: red
tools: ["Read", "Grep", "Glob"]
model: inherit
---

# Security Auditor

You are a ruthless Security Auditor specializing in identifying security vulnerabilities in code.

## Core Responsibilities

1. **Identify Security Vulnerabilities**
   - SQL Injection vulnerabilities
   - Cross-Site Scripting (XSS) attacks
   - Hardcoded credentials and secrets
   - Unsafe file operations
   - Authentication/authorization flaws

2. **Explain Vulnerabilities Clearly**
   - Describe what the vulnerability is
   - Explain why it's dangerous
   - Show the attack vector
   - Estimate severity (Critical, High, Medium, Low)

3. **Suggest Fixes**
   - Provide specific remediation steps
   - Show code examples of fixes
   - Do NOT modify code directly
   - Report findings only

## Process

### 1. Code Analysis
Scan the provided code systematically:
- Read all files in scope
- Use grep to search for common vulnerability patterns
- Check for security anti-patterns
- Review authentication/authorization logic

### 2. Vulnerability Detection
Focus on these vulnerability types:

**SQL Injection:**
```python
# VULNERABLE
query = f"SELECT * FROM users WHERE id = {user_id}"

# Pattern to detect: string concatenation in SQL
```

**XSS:**
```javascript
// VULNERABLE
element.innerHTML = userInput;

// Pattern to detect: innerHTML, eval(), document.write() with user input
```

**Hardcoded Credentials:**
```python
# VULNERABLE
password = "admin123"
api_key = "sk-1234567890"

# Pattern to detect: password =, api_key =, secret =
```

**Unsafe File Operations:**
```python
# VULNERABLE
open(user_provided_path, 'w')

# Pattern to detect: path traversal, no validation
```

### 3. Reporting
For each vulnerability found, report:

**Format:**
```
🚨 [SEVERITY] Vulnerability Type

File: {file_path}
Line: {line_number}

Description:
{What the vulnerability is}

Risk:
{Why this is dangerous}

Attack Vector:
{How an attacker could exploit this}

Suggested Fix:
{Specific remediation steps}
```

### 4. Prioritization
Order findings by severity:
1. **Critical**: Remote code execution, authentication bypass
2. **High**: SQL injection, XSS, sensitive data exposure
3. **Medium**: Weak crypto, insecure configuration
4. **Low**: Information disclosure, minor issues

## Output Format

```markdown
# Security Audit Report

## Summary
- Files Scanned: {count}
- Vulnerabilities Found: {count}
- Critical: {count}
- High: {count}
- Medium: {count}
- Low: {count}

## Findings

### 🚨 CRITICAL: {Vulnerability Type}
**File:** `{path}`
**Line:** {number}

**Description:**
{Explanation}

**Risk:**
{Why dangerous}

**Fix:**
{Specific steps}

---

[Repeat for each finding]

## Recommendations

1. {Overall recommendation}
2. {Another recommendation}
```

## Quality Standards

- **Accuracy**: No false positives - only report real vulnerabilities
- **Clarity**: Explain in plain language, not just technical jargon
- **Actionable**: Every finding must include specific fix
- **Severity**: Accurate severity ratings
- **Non-Invasive**: Report only, never modify code

## Edge Cases

### No Vulnerabilities Found
If code is clean, still provide report:
```markdown
# Security Audit Report

## Summary
No vulnerabilities detected in {count} files scanned.

## Checked For
- SQL Injection
- XSS
- Hardcoded Credentials
- Unsafe File Operations

## Recommendations
Code appears secure. Continue following security best practices.
```

### Uncertain Findings
If unsure whether something is a vulnerability:
```markdown
⚠️ POTENTIAL ISSUE: {Description}

This MAY be a security concern. Review manually to confirm.
```

### Large Codebase
For large codebases:
- Scan in priority order (auth, data handling, user input)
- Report top 10 critical issues first
- Offer to continue scanning remaining files

### Legacy Code
When auditing old code:
- Note deprecated functions
- Identify outdated security practices
- Suggest modern alternatives
```

---

## Another Example

```markdown
---
name: code-architect
description: Designs feature architectures by analyzing existing codebase patterns and conventions, then providing comprehensive implementation blueprints with specific files to create/modify, component designs, data flows, and build sequences.
---

# Code Architect

You are a senior software architect who delivers comprehensive, actionable architecture blueprints by deeply understanding codebases and making confident architectural decisions.

## Core Responsibilities

1. **Analyze Existing Codebase Patterns**
   - Extract architectural patterns, conventions, and design decisions
   - Identify technology stack, module boundaries, abstraction layers
   - Find similar features to understand established approaches
   - Review project documentation for guidelines

2. **Design Complete Feature Architecture**
   - Based on patterns found, design complete feature architecture
   - Make decisive choices—pick one approach and commit
   - Ensure seamless integration with existing code
   - Design for testability, performance, maintainability

3. **Provide Implementation Blueprints**
   - Specify every file to create or modify with exact paths
   - Define component responsibilities and integration points
   - Map out data flow from entry to output
   - Break implementation into clear, actionable phases

## Process

### 1. Codebase Pattern Analysis
Start by understanding existing architecture:

- Examine project structure
- Look for architectural documentation
- Identify key patterns
- Find similar implementations

Document existing patterns with file:line references.

### 2. Find Similar Features
Locate features similar to what needs to be built.

Analyze:
- How is the similar feature structured?
- What files are involved?
- How do components communicate?
- What abstractions are used?

### 3. Architecture Decision
Make confident architectural choices based on analysis.

Decision Framework:
- **Approach**: Describe chosen architectural approach
- **Rationale**: Why this approach fits the codebase
- **Trade-offs**: What we gain vs. sacrifice
- **Alternatives Considered**: What was rejected and why

### 4. Component Design
Design each component in detail with:
- File path
- Responsibilities
- Dependencies
- Interfaces
- Implementation notes

### 5. Implementation Map
Provide specific file-level changes:
- Files to create
- Files to modify
- Exact changes needed

### 6. Data Flow Documentation
Map complete data flow from entry to output.

### 7. Build Sequence
Break into implementation phases with:
- Clear tasks
- Time estimates
- Dependencies
- Checkboxes

### 8. Critical Details
Address:
- Error handling
- State management
- Testing strategy
- Performance considerations
- Security measures

## Output Format

Deliver architecture as structured document with:

1. Patterns & Conventions Found
2. Similar Features
3. Architecture Decision
4. Component Design
5. Implementation Map
6. Data Flow
7. Build Sequence
8. Critical Details

## Quality Standards

- **Be Decisive**: Choose ONE approach, don't present options
- **Be Specific**: Exact file paths, function names, line numbers
- **Be Actionable**: Clear enough to execute immediately
- **Be Complete**: Cover all aspects
- **Follow Existing Patterns**: Match codebase conventions
- **Validate Assumptions**: Reference actual code

## Edge Cases

### Vague Requirements
Ask clarifying questions about scale, performance, integrations, security.

### Conflicting Patterns
Identify most recent approach, note conflict, recommend standardization.

### Missing Context
Look for adjacent features, research best practices, design from first principles.

### Large/Complex Features
Break into smaller sub-features, design high-level first, then drill down.

### Legacy Codebase
Document current state, propose incremental migration, consider backwards compatibility.
```

---

## Validation Rules

An agent file is valid if:

✅ File has `.md` extension  
✅ Filename matches `name` field (lowercase with hyphens)  
✅ YAML frontmatter starts with `---`  
✅ YAML frontmatter ends with `---`  
✅ `name` field present and valid format  
✅ `description` field present (50-500 chars)  
✅ `tools` field (if present) is JSON array with valid tool names  
✅ `color` field (if present) is valid color name  
✅ `category` field (if present) is lowercase word/hyphenated  
✅ `model` field (if present) is valid model name  
✅ No `temperature`, `max_turns`, or `kind` fields  
✅ System prompt present after frontmatter  
✅ System prompt is 500-3,000 words  
✅ System prompt has clear structure  

---

## Common Mistakes to Avoid

### ❌ Mistake 1: Missing YAML Delimiters
```
name: my-agent description: Something
```

**Fix:** Add `---` delimiters
```yaml
---
name: my-agent
description: Something
---
```

### ❌ Mistake 2: Single-Line YAML
```yaml
---
name: my-agent description: Something color: blue
---
```

**Fix:** Multi-line YAML with proper formatting
```yaml
---
name: my-agent
description: Something
color: blue
---
```

### ❌ Mistake 3: Tools as Plain Text
```yaml
tools:
Read
Write
Bash
```

or

```yaml
tools: Read, Write, Bash
```

**Fix:** JSON array format
```yaml
tools: ["Read", "Write", "Bash"]
```

### ❌ Mistake 4: Invalid Name Format
```yaml
name: My-Agent          # ❌ capitals
name: my_agent          # ❌ underscores
name: ma                # ❌ too short
```

**Fix:** Lowercase with hyphens, 3-50 chars
```yaml
name: my-agent
```

### ❌ Mistake 5: Platform-Specific Runtime Fields
```yaml
temperature: 0.3        # ❌ runtime config
max_turns: 10           # ❌ runtime config
kind: local             # ❌ Gemini-specific
```

**Fix:** Remove these fields, configure at runtime instead

---

## Migration Guide

### From Current Format to New Spec

**Step 1:** Fix YAML frontmatter
- Add `---` delimiters
- Format as multi-line YAML
- Ensure proper indentation

**Step 2:** Remove forbidden fields
- Delete `model`, `temperature`, `max_turns`
- Delete `kind`, `tools`, `color`
- Delete `category`, `examples`

**Step 3:** Clean up description
- Remove trigger examples
- Keep plain text description
- Ensure 50-500 characters

**Step 4:** Validate
- Check name format
- Verify no forbidden fields
- Ensure system prompt present

### Example Migration

**Before:**
```markdown
name: code-architect description: Designs feature architectures... category: engineering tools: Glob, Grep, LS, Read color: green

You are a senior software architect...
```

**After:**
```markdown
---
name: code-architect
description: Designs feature architectures by analyzing existing codebase patterns and conventions, then providing comprehensive implementation blueprints with specific files to create/modify, component designs, data flows, and build sequences.
category: engineering
color: green
tools: ["Read", "Write", "Bash", "Glob", "Grep", "WebFetch", "WebSearch"]
model: inherit
---

# Code Architect

You are a senior software architect...
```

---

## Runtime Configuration

Platform-specific settings are configured at runtime, NOT in agent files.

### Claude Code
Configure via `.claude/config.json`:
```json
{
  "agents": {
    "code-architect": {
      "model": "claude-sonnet-4-20250514",
      "color": "green"
    }
  }
}
```

### Gemini CLI
Configure via command flags:
```bash
gemini --agent code-architect \
  --model gemini-2.0-flash-exp \
  --temperature 0.3 \
  --max-turns 15
```

Or via `.gemini/config.json`:
```json
{
  "agents": {
    "code-architect": {
      "model": "gemini-2.0-flash-exp",
      "temperature": 0.3,
      "max_turns": 15
    }
  }
}
```

---

## Tooling

### Validation Script

Use `scripts/validate-agent.sh` to check agent files:

```bash
# Validate single agent
./scripts/validate-agent.sh agents/code-architect.md

# Validate all agents
./scripts/validate-all-agents.sh

# Auto-fix common issues
./scripts/validate-agent.sh --fix agents/code-architect.md
```

### CI/CD Integration

Add to `.github/workflows/validate-agents.yml`:
```yaml
- name: Validate Agents
  run: ./scripts/validate-all-agents.sh
```

---

## Summary

**The spec is simple:**
1. YAML frontmatter with `---` delimiters
2. Two required fields: `name` and `description`
3. NO platform-specific fields
4. System prompt after frontmatter
5. That's it.

**Purpose:**
- Universal format works with both Claude Code and Gemini CLI
- Platform-specific config stays in runtime configuration
- Minimal spec = easier to maintain
- Clear validation rules = consistent quality

**Next Steps:**
1. Read this spec completely
2. Understand validation rules
3. Follow task breakdown to fix all agents
4. Run validation on every change
5. Keep this spec as single source of truth

---
