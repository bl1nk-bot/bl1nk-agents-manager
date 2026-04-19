# TODO: bl1nk-agents

## [2026-04-18] CI/CD & Context Stabilization (v1.7.1)

- [x] **Modernize GitHub Actions**
  - [x] Replace deprecated `actions-rs/toolchain` with `dtolnay/rust-toolchain` (Fix Node 20 warning)
  - [x] Update CodeQL Action from v3 to v4
  - [x] Fix non-existent `actions/checkout@v6` back to `@v4`
- [x] **Auto-Labeling System**
  - [x] Create `.github/labels.json` with presets and patterns
  - [x] Implement `scripts/edit-issue-labels.sh` for auto-labeling logic
  - [x] Create `.github/workflows/auto-labels.yml` for automation
- [x] **Project Cleanup**
  - [x] Merge PR #29 to remove outdated files and incorrectly placed configurations
  - [x] Sync `todo.md` with `conductor` and `.omg/state/taskboard.md`

## [2026-04-17] Split Metadata & Discovery Upgrade (v1.7.0)

- [x] ID 9: Port Skill Discovery from TS to Rust
- [x] ID 10: Implement Split Metadata Architecture (Clean 4 fields in .md)
- [x] ID 11: Unified Registry & Schema v1.7.0 Enforcement
- [x] ID 12: Organize Conductor Tracks & Archive old work
- [x] ID 13: Standardize Markdown Formatting (markdownlint-cli2)
- [x] ID 14: Track: Context Management Implementation (Current Focus)
  - [x] Phase 1: Context Data Structures (21 tests passing)
  - [x] Phase 2: JSON Persistence
  - [x] Phases 3-5 Plan: Created at conductor/tracks/context_management_20260417/phases_3_5_plan.md
- [~] **Tool Compaction TS→Rust** (Phase 2)
  - [x] Task 1: Module structure
  - [x] Task 2: index_tool_calls
  - [x] Task 3: find_pending_compaction_candidates
  - [~] Task 4: compact functions (Implementation stage)

## [2026-04-12] Codebase Stabilization (Archived Context)

- [x] ID 0-8: Stabilization tasks (verified 41 tests passing, 0 warnings)
- [ ] ID 9: Final Documentation & Handoff (Update README/SESSION_CONTEXT)
