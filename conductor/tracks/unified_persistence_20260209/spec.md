# Specification: Unified Persistence (System Cleanup)

## 1. Overview
Currently, many hooks and modules (e.g., `ralph_loop`, `session_recovery`) implement their own `storage.rs` logic. This track aims to unify these into a single, high-performance persistence layer in `src/persistence/mod.rs` to ensure consistency, prevent race conditions, and clean up the codebase.

## 2. Functional Requirements
- **Centralized API:**
    - Create a unified trait or struct for JSON/TOML persistence.
    - Support both Local (Project-specific) and Global (User-specific) storage.
- **Migration:**
    - Migrate `ralph_loop` storage to the new system.
    - Migrate `session_recovery` storage.
    - Migrate `discovery_cache` (from Track 1) to this system.
- **Thread Safety:**
    - Use appropriate locking (e.g., `Arc<RwLock>`) to prevent file corruption during concurrent access.

## 3. Non-Functional Requirements
- **Standardization:** All system files must be stored in standardized paths (e.g., `~/.config/bl1nk-agents-manager/`).
- **Robustness:** Implement automatic backups or atomic write operations (write-to-tmp-then-rename).

## 4. Acceptance Criteria
- `src/persistence/` directory exists.
- At least two existing hooks are refactored to use the new persistence layer.
- No more scattered `storage.rs` files in the core modules.
