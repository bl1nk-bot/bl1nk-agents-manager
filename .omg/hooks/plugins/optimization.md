# Optimization Plugin (P2)

Suggest performance and efficiency improvements.

**Fail Policy:** Fail-open | **Timeout:** 600ms | **Debounce:** 500ms

## Triggers

- Sequential operations that could be parallel → SUGGEST
- Unnecessary allocations/clones → SUGGEST
- Missing caching opportunities → SUGGEST
- Bundle size increase → SUGGEST
- N+1 query patterns → SUGGEST

## Rules

- rust-schemars-datetime
- bumpversion-and-changelog
