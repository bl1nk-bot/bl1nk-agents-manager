# OmG Hooks

Hook system for runtime event interception and quality enforcement.

## Configuration

Profile: `balanced` | Config: [../state/hooks.json](../state/hooks.json)

## Event Lanes

| Lane | Priority | Fail Policy |
|------|----------|-------------|
| `P0-safety` | Highest | Fail-closed |
| `P1-quality` | Medium | Fail-open |
| `P2-optimization` | Lowest | Fail-open |

## Plugins

- [safety](plugins/safety.md) — P0
- [quality](plugins/quality.md) — P1
- [optimization](plugins/optimization.md) — P2

## Plugin Contract

```typescript
onHookEvent(event: HookEvent, sdk: HookSDK): Promise<HookResult>
```

### Event Envelope

`event`, `source`, `session_id`, `task_id?`, `lane`, `subagent?`, `termination_reason?`, `metadata`

### SDK

`log()`, `state.get()`, `state.set()`

### Return

`verdict: 'pass' | 'warn' | 'block'`, `reason?`, `action?: 'continue' | 'reroute' | 'stop'`

## Guardrails

1. Side-effect hooks off for worker sessions
2. Fail-open for P2-optimization
3. Fail-closed for P0-safety violations
4. Blocked continuations re-enter safety lane
5. Terminal outcomes recorded once per agent turn
6. No volatile churn in persisted state

## Validation

`/omg:hooks-validate`
