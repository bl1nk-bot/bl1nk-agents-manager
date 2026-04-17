# Hooks Validation Report

**Date:** 2026-04-12
**Profile:** balanced
**Config:** `.omg/state/hooks.json`
**Commit:** `6225afc`

---

## Validation Result

| Field | Value |
|-------|-------|
| **overall** | ✅ pass (with minors) |
| **profile** | balanced |
| **lifecycle** | ✅ symmetric |
| **critical** | 0 |
| **major** | 1 |
| **minor** | 3 |

---

## Findings

| Severity | Finding | Evidence | Fix |
|----------|---------|----------|-----|
| major | No explicit event→lane mapping in config | `hooks.json` has `native_events` list but no `lane` field per event | Add `"event_lanes"` map to config for determinism |
| minor | No idempotency key configuration | No `idempotency` field in config; repeated events could trigger duplicate hooks | Add `"idempotency_key_fields": ["event", "task_id", "session_id"]` |
| minor | No explicit high-risk hook confirmation policy | `team_safety` disables side-effects for workers but doesn't require user confirmation for P0 hooks | Add `"require_confirmation_for": ["P0-safety"]` |
| minor | Derived signals have no lane assignment | 6 derived signals (context-drift, risk-spike, etc.) have no explicit lane routing | Map derived signals to lanes (recommend: risk-spike→P0, loop-stall→P1, others→P2) |

---

## Detailed Checks

### 1. Event → Lane Mapping

- **Status:** ⚠️ Implicit
- Native events exist but no explicit lane assignment in config
- Events likely route by plugin trigger logic, not config-driven
- **Risk:** Nondeterminism if multiple plugins claim same event

### 2. Cyclic Trigger Chains

- **Status:** ✅ No cycles detected
- `continuation_reenters_safety_lane: true` prevents loops
- `token-burst: false` disables a potential burst cycle
- Flow: `pre-verify → post-verify → checkpoint-save` is linear, not cyclic

### 3. Ordering Determinism

- **Status:** ⚠️ Partial
- Lifecycle order is deterministic (session-start → ... → session-stop)
- Within-lane ordering is undefined (no priority field)
- **Recommendation:** Add `priority` field to plugins for deterministic intra-lane order

### 4. Timeout & Debounce Budgets

- **Status:** ✅ Reasonable
| Lane | Timeout | Debounce | Assessment |
|------|---------|----------|------------|
| P0-safety | 400ms | 0ms | ✅ Tight, appropriate for safety |
| P1-quality | 800ms | 200ms | ✅ Relaxed for thorough checks |
| P2-optimization | 600ms | 500ms | ✅ Higher debounce reduces noise |

### 5. Lifecycle Symmetry

- **Status:** ✅ Symmetric
- `before_after_symmetry: true` — each entry has matching exit
- `continuation_reenters_safety_lane: true` — blocked → P0 → retry
- Terminal states: `completed`, `blocked`, `stopped` — exhaustive
- No double-fire: terminal outcomes recorded once per agent turn

### 6. Team-Safety Policy

- **Status:** ✅ Configured
- `disable_side_effects_for_worker_sessions: true` — workers can't trigger side effects
- **Gap:** No explicit user confirmation requirement for high-risk hooks

### 7. Plugin Configuration

- **Status:** ✅ All 3 plugins enabled and mapped
| Plugin | Lane | Enabled | Status |
|--------|------|:-------:|--------|
| safety | P0 | ✅ | Active |
| quality | P1 | ✅ | Active |
| optimization | P2 | ✅ | Active |

---

## Safe-to-Run Decision

| Decision | Value |
|----------|-------|
| **Safe to run** | ✅ Yes |
| **Rationale** | No critical issues. Lifecycle symmetry confirmed. Team-safety policy active. Timeout budgets reasonable. Minor gaps are non-blocking and can be addressed in next iteration. |

---

## Recommended Fixes (Priority Order)

1. **[major]** Add explicit `event_lanes` map to `hooks.json`
2. **[minor]** Add `idempotency_key_fields` to prevent duplicate triggers
3. **[minor]** Add `require_confirmation_for` for P0 hooks
4. **[minor]** Map derived signals to lanes

---

*Next: Apply recommended fixes, then re-validate*
