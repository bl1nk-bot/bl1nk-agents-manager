# Quality Plugin (P1)

Enforce code quality standards and project conventions.

**Fail Policy:** Fail-open | **Timeout:** 800ms | **Debounce:** 200ms

## Triggers

- Missing tests for new code → WARN
- Conventional commit format violation → WARN
- Missing rustdoc for public APIs → WARN
- Coverage below 90% for new code → WARN
- Linter warnings (clippy, fmt) → WARN

## Rules

- conventional-commit-linting
- parallel-ci-checks
- use-ask-user-always (alwaysApply)
