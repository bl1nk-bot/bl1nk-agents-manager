# 🤖 Bl1nk Agents Manager

![Version](https://img.shields.io/badge/version-0.2.0-blue.svg)
![License](https://img.shields.io/badge/license-MIT-green.svg)
![Status](https://img.shields.io/badge/status-active-success.svg)
![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)

> **A sophisticated multi-agent orchestration system with 48+ specialized agents for Gemini CLI**

Bl1nk Agents Manager transforms the generic Gemini CLI assistant into a specialized workforce of AI agents. It provides a comprehensive system for managing, switching, and maintaining a library of **System Agents**—specialized prompts designed for specific domains like Architecture, Coding, Writing, Research, and more.

---

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
## 📖 Table of Contents

- [Features](#-features)
- [Quick Start](#-quick-start)
- [Available Agents](#-available-agents)
- [Architecture](#-architecture)
- [Hooks System](#-hooks-system)
- [Development](#-development)
- [Contributing](#-contributing)
- [License](#-license)

---

## ✨ Features

- **📚 Extensive Agent Library**: Access 48+ pre-built, high-quality agents including Software Architect, Code Generator, Cloudflare specialist, Code Reviewer, and more.
- **⚡ High-Performance Core**: Built on Rust with Tokio async runtime for concurrent operations.
- **🪝 Advanced Hook System**: 35+ hooks for context injection, monitoring, recovery, and automation.
- **🔄 Dual-Protocol Support**: MCP (Model Context Protocol) for CLI integration and internal agent communication.
- **📦 Modular Architecture**: Clean separation between agents, hooks, filesystem, search, and session management.
- **🎯 Specialized Workflows**: Dedicated agents for code review, architecture planning, research, documentation, and full-stack development.
- **🔧 Extensible Design**: Easy addition of custom agents and hooks with comprehensive metadata.

---

## 🚀 Quick Start

### Installation

1. Navigate to your Gemini CLI extensions directory:

   ```bash
   cd ~/.gemini/extensions/
   ```

2. Clone this repository:

   ```bash
   git clone https://github.com/billlzzz18/bl1nk-agents-manager.git
   cd bl1nk-agents-manager
   ```

3. Build the project:

   ```bash
   just build
   ```

### Usage

List all available agents:

```bash
/system-agent
```

Get detailed information about an agent:

```bash
/system-agent:info architect
```

Switch to a specific agent:

```bash
/system-agent:switch pirate
```

Create a new agent:

```bash
/system-agent:new
```

---

## 🧠 Available Agents

### Engineering & Development

| Agent | Description |
|-------|-------------|
| [Architect](agents/architect.md) | Software architecture and design planning |
| [Code Generator](agents/code-generator.md) | Rapid, clean code generation |
| [Code Reviewer](agents/code-reviewer.md) | Bug detection and code quality |
| [Code Explorer](agents/code-explorer.md) | Deep codebase analysis |
| [Code Architect](agents/code-architect.md) | Feature architecture design |
| [Cloudflare](agents/cloudflare.md) | Cloudflare Workers and Agents |
| [Fullstack Dev](agents/fullstack-dev.md) | Full-stack application development |
| [Orchestrator](agents/orchestrator.md) | Task delegation and routing |

### Research & Analysis

| Agent | Description |
|-------|-------------|
| [Codebase Analyzer](agents/codebase-analyzer.md) | Implementation detail analysis |
| [Codebase Locator](agents/codebase-locator.md) | File and component discovery |
| [Codebase Pattern Finder](agents/codebase-pattern-finder.md) | Similar implementation search |
| [Research Analyzer](agents/research-analyzer.md) | Research document analysis |
| [Thoughts Analyzer](agents/thoughts-analyzer.md) | Deep research on topics |
| [Web Search Researcher](agents/web-search-researcher.md) | Web content research |

### Documentation & Planning

| Agent | Description |
|-------|-------------|
| [Docbot Pro](agents/docbot-pro.md) | Enterprise documentation |
| [Docs Researcher](agents/docs-researcher.md) | Library documentation |
| [Insight Documenter](agents/insight-documenter.md) | Technical breakthrough docs |
| [Plan Implementation Reviewer](agents/plan-implementation-reviewer.md) | Plan validation |

### Utilities & Tools

| Agent | Description |
|-------|-------------|
| [Agent Creator](agents/agent-creator.md) | Create new agents |
| [Command Creator](agents/command-creator.md) | Create Claude Code commands |
| [Skill Creator](agents/skill-creator.md) | Create new skills |
| [Skill Reviewer](agents/skill-reviewer.md) | Skill quality review |
| [Plugin Validator](agents/plugin-validator.md) | Plugin structure validation |
| [Task Management](agents/task-management.md) | Task tracking and context |
| [UI Engineer](agents/ui-engineer.md) | Frontend/UI development |

### Creative & Entertainment

| Agent | Description |
|-------|-------------|
| [Creative Writer](agents/creative-writer.md) | Poetry, prose, storytelling |
| [Pirate](agents/pirate.md) | Pirate dialect assistant |
| [Yoda](agents/yoda.md) | Yoda-speak assistant |
| [Shakespeare](agents/shakespeare.md) | Shakespearean English |
| [Cowboy](agents/cowboy.md) | Western dialect |
| [Gen Z](agents/gen-z.md) | Gen Z slang |
| [Comedian](agents/comedian.md) | Dad jokes |

---

## 🏗️ Architecture

```
bl1nk-agents-manager/
├── agents/                 # Agent definitions (48+ agents)
│   ├── *.md               # Agent system prompts
│   └── agents.json        # Agent registry
├── commands/               # CLI command definitions
│   ├── system-agent.toml   # Main /system-agent command
│   └── agent/              # Subcommands
├── crates/
│   ├── core/              # Core library
│   │   └── src/
│   │       ├── lib.rs     # Main library entry
│   │       ├── agents/    # Agent system (16 modules)
│   │       ├── hooks/     # Hook system (35+ hooks)
│   │       ├── mcp/       # MCP protocol
│   │       ├── session/   # Session management
│   │       ├── filesystem/ # File operations
│   │       ├── search/    # Conversation search
│   │       ├── projects/   # Project management
│   │       ├── adapters/   # Protocol adapters
│   │       ├── config/    # Configuration
│   │       ├── rpc/       # RPC handling
│   │       └── events/    # Event system
│   └── server/             # HTTP/Rocket server
├── skills/                 # AI skill definitions
├── scripts/                # Python management scripts
├── docs/                   # Documentation
└── justfile               # Build commands
```

### Core Components

1. **Agent System** (`crates/core/src/agents/`):
   - `register.rs`: Agent registry and management
   - `router.rs`: Intelligent agent routing
   - `executor.rs`: Task execution
   - `orchestrator.rs`: Multi-agent coordination
   - `expert.rs`, `researcher.rs`, `explorer.rs`: Specialized agents

2. **Hooks System** (`crates/core/src/hooks/`):
   - `todo_continuation_enforcer.rs`: Task continuity
   - `context_window_monitor.rs`: Memory monitoring
   - `directory_agents_injector.rs`: Context injection
   - `ralph_loop.rs`: Loop detection and recovery
   - `edit_error_recovery.rs`: Error recovery
   - 30+ more hooks for various purposes

3. **Session Management** (`crates/core/src/session/`):
   - Process lifecycle management
   - Backend session handling
   - Message routing

4. **Filesystem** (`crates/core/src/filesystem/`):
   - Cross-platform file operations
   - Git integration
   - Path normalization

---

## 🪝 Hooks System

Bl1nk includes 35+ hooks for advanced automation and monitoring:

### Context & Injection

| Hook | Purpose |
|------|---------|
| `directory_agents_injector` | Inject agent context from directory |
| `directory_readme_injector` | Inject README context |
| `compaction_context_injector` | Context compaction |
| `rules_injector` | Rule injection |
| `ralph_loop` | Loop detection and recovery |

### Monitoring & Recovery

| Hook | Purpose |
|------|---------|
| `context_window_monitor` | Memory usage monitoring |
| `session_recovery` | Session recovery |
| `anthropic_context_window_limit_recovery` | Claude context recovery |
| `empty_task_response_detector` | Detect empty responses |
| `edit_error_recovery` | Recover from edit errors |

### Task Management

| Hook | Purpose |
|------|---------|
| `todo_continuation_enforcer` | Enforce TODO completion |
| `category_skill_reminder` | Category/skill reminders |
| `task_resume_info` | Task resumption info |
| `start_work` | Work session startup |

### Development Tools

| Hook | Purpose |
|------|---------|
| `comment_checker` | Check for TODO comments |
| `tool_output_truncator` | Truncate long outputs |
| `thinking_block_validator` | Validate thinking blocks |
| `question_label_truncator` | Truncate question labels |

---

## 🛠️ Development

### Prerequisites

- Rust 1.70+
- Python 3.8+ (for management scripts)
- Gemini CLI

### Setup

```bash
# Clone the repository
git clone https://github.com/billlzzz18/bl1nk-agents-manager.git
cd bl1nk-agents-manager

# Install development tools
just setup

# Build the project
just build

# Run tests
just test
```

### Available Commands

```bash
just build          # Standard release build
just build-bundled  # Build with bundled PMAT
just run            # Run the binary
just dev            # Development mode with hot-reload
just test           # Run all tests
just check          # Quick compilation check
just fmt            # Format code
just clippy         # Run linter
just validate-agents # Validate agent files
just fix-agents     # Fix agent metadata
```

### Project Structure

- `crates/core/src/agents/mod.rs` - Agent system (16 modules)
- `crates/core/src/hooks/mod.rs` - Hook system (35+ hooks)
- `crates/core/src/mcp/mod.rs` - MCP orchestrator
- `crates/core/src/session/` - Session management
- `crates/core/src/filesystem/` - File operations
- `crates/core/src/search/` - Conversation search
- `crates/core/src/projects/` - Project management

---

## 🤝 Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Adding New Agents

1. Create a new `.md` file in `agents/` with YAML frontmatter
2. Add agent metadata to `agents/agents.json`
3. Run `just validate-agents` to verify

### Adding New Hooks

1. Create a new module in `crates/core/src/hooks/`
2. Implement the hook interface
3. Register in `crates/core/src/hooks/mod.rs`

---

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

**Built with ❤️ for the AI Agent Community.**

Maintained by [billlzzz18](https://github.com/billlzzz18) and contributors.
