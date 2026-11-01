# Rust Development Guide

This document defines the **mandatory standards** for all Rust projects in this repository.
All contributions **MUST** comply with this guide.

---

## 1. Core Principles

### 1.1 Safety & Correctness First
- Use Rust ownership, borrowing, and lifetimes correctly.
- Avoid `unsafe` unless absolutely required and explicitly documented.
- Prefer correctness and clarity over cleverness.

### 1.2 Idiomatic & Performant Rust
- Write idiomatic Rust following community best practices.
- Prefer zero-cost abstractions.
- Code must be readable without excessive comments.

---

## 2. Dependency Policy

### 2.1 Dependency Selection
- Minimize external dependencies.
- Do not add a dependency without clear justification.
- Prefer extending existing internal code over adding new crates.

### 2.2 Approved Baseline Dependencies
- `rand`
- `rayon`

Any additional dependency **MUST** be justified in the Pull Request.

### 2.3 Dependency Validation
- Run `cargo deny` to validate:
  - Licenses
  - Duplicate dependencies
  - Unapproved sources

---

## 3. Code Formatting & Style

### 3.1 Formatting
- Use `rustfmt` exclusively.
- Configuration (`rustfmt.toml`):
  ```toml
  max_width = 100
  ```

### 3.2 Naming Conventions
- Types: `PascalCase`
- Functions & variables: `snake_case`
- Constants: `SCREAMING_SNAKE_CASE`
- Modules: `snake_case`

### 3.3 Linting
- Use `clippy`.
- All warnings must be fixed:
  ```bash
  cargo clippy -- -D warnings
  ```

---

## 4. Error Handling
- Libraries: use `thiserror`
- Applications / binaries: use `anyhow`
- Do not panic in library code.
- Errors must be meaningful and actionable.

---

## 5. Documentation Standards

### 5.1 Rust Documentation
- Use `//!` for module-level documentation.
- Use `///` for public items.

Public APIs **MUST** include:
- Purpose
- `# Examples`
- `# Errors` (if applicable)
- `# Panics` (if applicable)

### 5.2 Accuracy
- Documentation must reflect actual behavior.
- Outdated documentation is considered a bug.

---

## 6. Testing Standards

### 6.1 Test Structure
- Unit tests: `mod tests` inside the module
- Integration tests: `tests/` directory

### 6.2 Coverage
- Target: **≥ 85% line coverage**
- Use:
  ```bash
  cargo llvm-cov
  ```

### 6.3 Ignored Tests
- `#[ignore]` tests MUST include:
  - Reason for being ignored
  - Plan to re-enable

---

## 7. Build & Tooling

### 7.1 Task Runner
- Use `just` with a `justfile` for common tasks.

### 7.2 Build Scripts
- Use `build.rs` only for:
  - Code generation
  - Compile-time configuration
- Keep build scripts minimal and deterministic.

---

## 8. Workflow Requirements

### 8.1 Development Workflow
1. Implement changes following this guide.
2. Update documentation if behavior changes.
3. Run:
   ```bash
   cargo fmt
   cargo clippy -- -D warnings
   cargo test
   ```
4. Commit only when all checks pass.

### 8.2 Debugging
- Debug using evidence, logs, and tests.
- Fix root causes, not symptoms.
- Add regression tests for every bug fix.

---

## 9. Consistency Rule (Non-Negotiable)

When editing existing code, **match the existing style**.
Consistency across the codebase is more important than personal preference.