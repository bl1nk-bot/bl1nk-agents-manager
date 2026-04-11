# Fix Branch - Task Checklist

## Option 1: Cleanup Old Config Files

### 1. Delete Old Config Files
- [ ] Delete `config.toml` (old test config in root)
- [ ] Delete `config_test.toml` (old test config in root)
- [ ] Delete `Config.example.toml` (duplicate example, use `.config/config.example.toml`)

### 2. Update Test Integration Script
- [ ] Update `scripts/test_integration.py` to reference `.config/config.example.toml` instead of `config_test.toml`

### 3. Update Documentation References
- [ ] Update `docs/QUICKSTART.md` to reference `.config/config.example.toml`
- [ ] Update `docs/PROJECT_SUMMARY.md` to reference `.config/config.example.toml`

### 4. Verify All Changes
- [ ] Verify no references to old config files remain
- [ ] Verify all new config files are in `.config/`
- [ ] Run tests to ensure nothing is broken

### 5. Commit and Push
- [ ] Commit all changes with message: `fix: cleanup old config files and update references`
- [ ] Push to `fix` branch
- [ ] Create PR #22

---

## Files to Delete

| File | Path | Reason |
|------|------|--------|
| `config.toml` | Root | Old test config |
| `config_test.toml` | Root | Old test config |
| `Config.example.toml` | Root | Duplicate (use `.config/`) |

## Files to Update

| File | Change |
|------|--------|
| `scripts/test_integration.py` | Change `config_test.toml` to `.config/config.example.toml` |
| `docs/QUICKSTART.md` | Change references to `.config/config.example.toml` |
| `docs/PROJECT_SUMMARY.md` | Change references to `.config/config.example.toml` |

## Files to Keep

| File | Path | Reason |
|------|------|--------|
| `config.example.toml` | `.config/` | Main example config |
| `config.example.json` | `.config/` | JSON format example |
| `config.example.yaml` | `.config/` | YAML format example |
| `schema-config.json` | `.config/` | Unified config schema |
| `schema-agent.json` | `.config/` | Agent spec schema |
| `gemini-extension.json` | Root | Extension config |
| `rustfmt.toml` | Root | Rust formatting |

---

*Created: 2026-04-11*
