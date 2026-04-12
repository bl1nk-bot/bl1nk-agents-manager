# Conductor Workflow & Project Structure

## Conductor Files
```
conductor/
├── product.md          # Product definition
├── tech-stack.md       # Technology stack
├── workflow.md         # Development workflow
├── tracks.md           # Tracks registry
└── tracks/
    └── registry_knowledge_backbone_20260412/
        ├── spec.md     # Track specification
        └── plan.md     # Implementation plan
```

## Active Track
**Unified Registry, Monitoring & Honesty Layer**
- Phase 1: Foundation (Task 1.1 ✅ complete, Task 1.2 next)
- Task 1.1: Define Unified Registry Schema → Commit `0780c25`
- Total phases: 8 (Foundation → Documentation & Cleanup)

## Development Workflow (จาก workflow.md)
1. Read Context (todo.md, plan.md)
2. Select Task
3. Write Failing Tests (Red Phase)
4. Implement to Pass Tests (Green Phase)
5. Refactor (Optional)
6. Verify Coverage (>90%)
7. Commit Code
8. Update Plan
9. User Verification

## Commit Convention
```
type(scope): description
```
Types: feat, fix, docs, style, refactor, perf, test, chore, security, conductor, ci, build, revert

## Key Makefile Targets
```bash
make parallel          # Fast: fmt + clippy + test พร้อมกัน
make review [TARGET]   # Code review
make bump-patch        # Bump version
make changelog         # Generate CHANGELOG
make security-check    # Security audit
```

## Scripts
| Script |做什么 |
|--------|------|
| `scripts/parallel-check.sh` | รัน checks พร้อมกัน |
| `scripts/bumpversion.sh` | Bump version + tag |
| `scripts/generate-changelog.sh` | สร้าง CHANGELOG |
| `scripts/commitlint.sh` | ตรวจสอบ commit messages |
| `scripts/update-security.sh` | Security audit |
| `scripts/review.sh` | Code review wrapper |
