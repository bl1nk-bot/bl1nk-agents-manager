# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.2.x   | :white_check_mark: |
| 0.1.x   | :warning:          |
| < 0.1   | :x:                |

## Reporting a Vulnerability

We take the security of BL1NK Agents Manager seriously. If you discover a security vulnerability, please follow these steps:

### DO NOT
- ❌ Open a public GitHub issue
- ❌ Discuss the vulnerability in public forums
- ❌ Share the vulnerability details with others

### DO
- ✅ Email us at: **security@bl1nk.dev** (if available)
- ✅ Open a [GitHub Security Advisory](https://github.com/billlzzz18/bl1nk-agents-manager/security/advisories/new)
- ✅ Include as much detail as possible:
  - Type of vulnerability (e.g., XSS, injection, auth bypass)
  - Affected component/version
  - Steps to reproduce
  - Potential impact
  - Suggested fix (if any)

## Response Timeline

| Stage | Expected Time |
|-------|---------------|
| Acknowledgment | Within 48 hours |
| Initial Assessment | Within 1 week |
| Fix Development | Within 2-4 weeks |
| Public Disclosure | After fix is released |

## Security Best Practices

### For Users

1. **Keep Dependencies Updated**
   ```bash
   # Check for outdated dependencies
   cargo outdated

   # Update to latest versions
   cargo update
   ```

2. **Use Latest Version**
   Always use the latest version of BL1NK Agents Manager to get security patches.

3. **Review Agent Configurations**
   - Never hardcode secrets in agent config files
   - Use environment variables for sensitive data
   - Review permission policies before deploying

4. **Enable Rate Limiting**
   Configure rate limits for all agents to prevent abuse:
   ```toml
   [agents.qwen-coder.rate_limit]
   requests_per_minute = 60
   requests_per_day = 2000
   ```

### For Developers

1. **No Secrets in Code**
   - Never commit API keys, passwords, or tokens
   - Use `.env` files (add to `.gitignore`)
   - Use secret scanning tools

2. **Validate All Input**
   - Validate agent configurations
   - Sanitize user inputs
   - Use type-safe parsing

3. **Minimize Unsafe Code**
   - Avoid `.unwrap()` in production code
   - Use `?` operator for error propagation
   - Minimize `unsafe` blocks

4. **Run Security Audits**
   ```bash
   # Install cargo-audit
   cargo install cargo-audit

   # Run security audit
   cargo audit
   ```

5. **Use Permission System**
   - Configure permission tiers appropriately
   - Review rule_parser.rs for custom policies
   - Enable hook system for additional validation

## Known Security Features

| Feature | Status | Description |
|---------|--------|-------------|
| Permission Tiers | ✅ Active | Default/User/Admin/Extension/Workspace |
| Rate Limiting | ✅ Active | Per-agent RPM/RPD throttling |
| Hook System | ✅ Active | Pre/Post tool validation |
| Rule Parser | ✅ Active | Regex-based command filtering |
| Input Validation | ✅ Active | Schema-based config validation |
| Secret Scanning | 🟡 Planned | Automated secret detection in CI |

## Audit History

| Date | Tool | Result |
|------|------|--------|
| 2026-04-12 | cargo audit | Not run (cargo-audit not installed) |
| - | - | - |

> Run `make security-check` to perform the latest security audit.

## Third-Party Dependencies

All dependencies are managed via `Cargo.toml` and `Cargo.lock`. We use:
- **serde** for type-safe serialization
- **tokio** for async runtime
- **clap** for CLI argument parsing
- **tracing** for structured logging

Regular dependency audits should be performed using:
```bash
cargo audit
cargo outdated
```

## Security Changelog

| Date | Change | Type |
|------|--------|------|
| 2026-04-12 | Initial security policy | Documentation |
| 2026-04-12 | Add security update script | Tooling |
| 2026-04-12 | Add commitlint for message validation | Process |

---

**Last Updated:** 2026-04-12
**Next Review:** 2026-05-12
