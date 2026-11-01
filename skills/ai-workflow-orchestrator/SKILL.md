---
name: ai-workflow-orchestrator
description: Automated workflow for systematic AI-User collaboration in monorepo tasks.
---

Ensures correct process order, decision logging, and error prevention.
metadata:
  version: 1.0.0
---

# AI Workflow Orchestrator

**Purpose**: Automate the 3-step workspace orchestration process with decision logging, verification, and error
prevention.

---

## 🎯 Core Process

### TRIGGER IDENTIFICATION

When user sends request:
1. **Parse Request**: What's the trigger?
   - Cleanup? → Use cleanup-workflow
   - Push code? → Use git-workflow
   - Add skill? → Use skill-intake-workflow
   - Fix issue? → Use issue-workflow

2. **Ask Clarification** if unclear
3. **Log Decision**: Capture trigger in decision log

### STEP 1: DOCUMENT SYNCHRONIZATION

- [  ] Update README.md with changes
- [  ] Update docs/* with new information
- [  ] Check cross-references
- [  ] Verify all links work
- **Log**: What docs changed and why

### STEP 2: SECURITY & HYGIENE AUDIT

- [  ] Run git status check
- [  ] Verify no sensitive files
- [  ] Verify no cache dirs
- [  ] Verify no duplicate files
- [  ] Run custom validation rules
- **Log**: Audit results and any issues found

### STEP 3: GIT SYNCHRONIZATION

- [  ] Verify working tree clean
- [  ] Create descriptive commit message
- [  ] Include decision rationale in commit
- [  ] Push to remote
- **Log**: Commit hash and success/failure

---

## 🔍 ERROR DETECTION & HANDLING

### Pre-Commit Validation

```
IF .gitignore violations detected:
  → Stop, log issue, ask user
  
IF Step 1 not completed:
  → Stop, log missing docs
  
IF sensitive files staged:
  → Stop, prevent push
```

### After Completion Validation

```
IF push failed:
  → Log error, suggest fix
  
IF validation failed:
  → Rollback, document issue
  
IF all passed:
  → Success, close decision log
```

---

## 📋 DECISION LOG FORMAT

Each action creates log entry:

```yaml
Decision_ID: DEC-20241225-001
Date: 2024-12-25 14:00
Trigger: User request or automated detection
Request: Exact user message or trigger source
Analysis: |
  - What was the situation?
  - What was missing?
  - What are the options?
Decision: What was chosen and why?
Action: 
  - Specific commands or steps
  - Which skill used
  - What was verified
Outcome: 
  - ✅ Success / ⚠️ Partial / ❌ Failed
  - Metrics (commits, files changed, etc)
Issues: 
  - What went wrong?
  - What's the fix?
LessonsLearned:
  - What to do next time
  - New rules/checks needed
NextAction: What's the follow-up?
References:
  - Previous decisions
  - Related skills
```

---

## 🔒 SECURITY CHECKPOINTS

**Never skip these**:

1. **Pre-Commit Check**
   - No .env files
   - No SSH keys
   - No credentials
   - No .gitconfig

2. **Duplicate Check**
   - No duplicate requirements files
   - No redundant rules
   - No conflicting configs

3. **Sensitive File Audit**
   - git ls-files grep security patterns
   - find . grep untracked sensitive files

---

## 📊 WORKFLOW STATE MACHINE

```
START
  ↓
[TRIGGER RECEIVED ]
  ├─ Clear? → Proceed
  └─ Unclear? → ASK_CLARIFICATION → Log Decision
  ↓
[ANALYZE REQUEST ]
  ├─ Cleanup? → cleanup-workflow
  ├─ Push? → git-workflow
  ├─ Add Skill? → skill-intake-workflow
  └─ Fix Issue? → issue-workflow
  ↓
[STEP 1: DOCUMENT SYNC ]
  ├─ Update docs
  ├─ Update README
  ├─ Verify links
  └─ Log: What changed
  ↓
[STEP 2: SECURITY AUDIT ]
  ├─ Check git status
  ├─ Verify patterns
  ├─ Run validations
  └─ Log: Audit results
  ↓
[STEP 3: GIT SYNC ]
  ├─ Commit with rationale
  ├─ Push to remote
  └─ Log: Commit hash
  ↓
[VALIDATION ]
  ├─ Success? → Log & Close
  ├─ Error? → Rollback & Log
  └─ Partial? → Document & Continue
  ↓
[DECISION LOG SAVED ]
  ↓
END
```

---

## 🎯 USE CASES

### Cleanup Request

```
Trigger: "ทำตามกและpush ไปgithub"
Process: 
  1. Analyze: Is request clear?
  2. Ask: "ต้องทำความสะอาดอะไร?"
  3. Execute: cleanup-workflow
  4. Document: What was deleted/fixed
  5. Audit: Security check
  6. Push: With decision log
```

### Issue Fix

```
Trigger: "ยังมี projects/ ต่างหากที่ยังเหลืออยู่"
Process:
  1. Detect: Root cause analysis
  2. Document: Why it was missed
  3. Fix: Remove + update .gitignore
  4. Audit: Verify no more leaks
  5. Push: Reference previous decision
  6. Update: Learn log for prevention
```

### New Skill Addition

```
Trigger: New skill files appear
Process:
  1. Validate: SKILL.md format check
  2. Analyze: Purpose and structure
  3. Document: Add to README
  4. Link: Cross-references
  5. Test: Can skill be invoked?
  6. Decision Log: Why this skill needed
```

---

## 📝 DECISION LOG LOCATION

```
docs/decision-logs/DECISIONS_YYYY-MM-DD.md
```

Reference format:
```

## Decision #N: [Title ]

**Date**: YYYY-MM-DD
**Trigger**: [What caused this? ]
**Status**: ✅ | ⚠️ | ❌
[Content ]
```

---

## 🚫 COMMON MISTAKES TO AVOID

1. ❌ Push before docs updated (violates Step 1)
2. ❌ No validation between steps
3. ❌ Missing decision log entry
4. ❌ Same error twice (no lesson logged)
5. ❌ Added rule without enforcement
6. ❌ New skill without validation

---

## ✅ VERIFICATION CHECKLIST

Before marking task complete:

- [  ] Step 1 completed: All docs updated
- [  ] Step 2 passed: All audits passed
- [  ] Step 3 done: Pushed to GitHub
- [  ] Decision log: Entry created
- [  ] No errors: Validation passed
- [  ] Lessons learned: Documented
