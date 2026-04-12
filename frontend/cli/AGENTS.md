# CLI KNOWLEDGE BASE
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
## OVERVIEW

CLI entry: `bunx bl1nk`. Interactive installer, doctor diagnostics. Commander.js + @clack/prompts.

## STRUCTURE

```
cli/
├── index.ts              # Commander.js entry (4 commands)
├── install.ts            # Interactive TUI (520 lines)
├── config-manager.ts     # JSONC parsing (664 lines)
├── types.ts              # InstallArgs, InstallConfig
├── model-fallback.ts     # Model fallback configuration
├── doctor/
│   ├── index.ts          # Doctor entry
│   ├── runner.ts         # Check orchestration
│   ├── formatter.ts      # Colored output
│   ├── constants.ts      # Check IDs, symbols
│   ├── types.ts          # CheckResult, CheckDefinition (114 lines)
│   └── checks/           # 14 checks, 21 files
│       ├── version.ts    # OpenCode + plugin version
│       ├── config.ts     # JSONC validity, Zod
│       ├── auth.ts       # Anthropic, OpenAI, Google
│       ├── dependencies.ts # AST-Grep, Comment Checker
│       ├── lsp.ts        # LSP connectivity
│       ├── mcp.ts        # MCP validation
│       ├── model-resolution.ts # Model resolution check
│       └── gh.ts         # GitHub CLI
├── run/
│   └── index.ts          # Session launcher
└── get-local-version/
    └── index.ts          # Version detection
```

## COMMANDS

| Command | Purpose |
|---------|---------|
| `install` | Interactive setup with provider selection |
| `doctor` | 14 health checks for diagnostics |
| `run` | Launch session with todo enforcement |
| `get-local-version` | Version detection and update check |

## DOCTOR CATEGORIES (14 Checks)

| Category | Checks |
|----------|--------|
| installation | opencode, plugin |
| configuration | config validity, Zod, model-resolution |
| authentication | anthropic, openai, google |
| dependencies | ast-grep, comment-checker, gh-cli |
| tools | LSP, MCP |
| updates | version comparison |

## HOW TO ADD CHECK

1. Create `src/cli/doctor/checks/my-check.ts`
2. Export `getXXXCheckDefinition()` factory returning `CheckDefinition`
3. Add to `getAllCheckDefinitions()` in `checks/index.ts`

## TUI FRAMEWORK

- **@clack/prompts**: `select()`, `spinner()`, `intro()`, `outro()`
- **picocolors**: Terminal colors for status and headers
- **Symbols**: ✓ (pass), ✗ (fail), ⚠ (warn), ℹ (info)

## ANTI-PATTERNS

- **Blocking in non-TTY**: Always check `process.stdout.isTTY`
- **Direct JSON.parse**: Use `parseJsonc()` from shared utils
- **Silent failures**: Return `warn` or `fail` in doctor instead of throwing
- **Hardcoded paths**: Use `getOpenCodeConfigPaths()` from `config-manager.ts`
