# Security Policy
## 📌 Project Status (Feb 7, 2026)

Bl1nk Agents Manager is in active development and is not feature‑complete yet.
This repo contains a working extension shell and a Rust core that is being
brought to feature parity with existing TypeScript logic.

**What works now**
- Extension manifest and Gemini CLI scaffolding are present.
- Core Rust modules exist for agents, hooks, MCP/ACP, sessions, and RPC.
- Command and documentation sets are present (currently being refreshed).

**In progress**
- TypeScript → Rust parity for large subsystems (background agents, config,
  ACP normalization).
- End‑to‑end session flows for Gemini/Codex/Qwen within a unified adapter.
- Validation of hook behavior and task orchestration across agents.

**Known gaps**
- Some Rust modules compile but are not fully wired end‑to‑end.
- Configuration loading/migration is still being aligned to actual runtime.
- Authentication flows for some CLIs still require manual steps.

**What to expect right now**
- You can explore the architecture, commands, and agent catalogs.
- Some workflows will still require manual setup or troubleshooting.

For a complete non‑developer overview, see `docs/PROJECT_STATUS.md`.
## Supported Versions

The following versions of Bl1nk Agents Manager are currently supported with security updates:

| Version | Supported |
|---------|-----------|
| 0.2.x   | ✅ Yes    |
| 0.1.x   | ✅ Yes    |
| < 0.1   | ❌ No     |

## Reporting a Vulnerability

We take security seriously. If you believe you have found a security vulnerability in Bl1nk Agents Manager, please report it responsibly.

### How to Report

1. **Do not** open a public issue on GitHub
2. Email your report to: [security@bl1nk.site](mailto:security@bl1nk.site)
3. Include a detailed description of the vulnerability
4. Include steps to reproduce the issue
5. Include any relevant screenshots or logs

### What to Expect

- We will acknowledge your report within 48 hours
- We will provide an initial assessment within 7 days
- We will keep you updated on progress
- We will not disclose the vulnerability publicly until a fix is released

## Security Best Practices

### Agent Sandboxing

- Agents run in isolated processes
- No shared memory between agents
- Clean shutdown on errors

### Input Validation

- JSON Schema validation for all MCP requests
- Path validation to prevent directory traversal
- Command whitelisting for agent execution

### Rate Limiting

- Per-agent request limits (60 requests/minute, 2000 requests/day)
- Configurable rate limits
- Automatic reset timers

### Data Privacy

- Local processing by default
- No data sent to external servers (except user-configured model providers)
- Conversation history stored locally

## Security Considerations

### Agent Trust

- Only use agents from trusted sources
- Review agent prompts before use
- Agents have access to filesystem based on user permissions

### Configuration

- Review configuration before deployment
- Use least privilege principles for agent permissions
- Regularly update to latest version

### Dependencies

- All dependencies are reviewed for security
- Rust crates are audited for known vulnerabilities
- Keep dependencies updated

## Related Security Resources

- [OWASP AI Security](https://owasp.org/www-project-ai-security/)
- [Model Context Protocol Security](https://modelcontextprotocol.io/)

---

Thank you for helping us keep Bl1nk secure!
