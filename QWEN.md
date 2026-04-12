# 🤖 Bl1nk Agents Manager Extension
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
## Project Overview

Bl1nk Agents Manager is a sophisticated multi-agent orchestration system that transforms generic AI assistants into specialized workforces. It provides 48+ specialized agents for domains like Architecture, Coding, Research, Documentation, and more.

The extension uses a hybrid architecture:

- **Rust crates** in `crates/` provide high-performance core functionality
  - `crates/core` - Core library with agent registry, routing, hooks, and configuration
  - `crates/server` - HTTP/Rocket server for advanced features
- **Agent definitions** in `agents/` contain 48+ system prompt files
- **Commands** in `commands/` define the CLI interface
- **Hooks** in `crates/core/src/hooks/` provide 35+ automation hooks

## Architecture

### Core Modules

- **Agent Registry** (`crates/core/src/agents/register.rs`): Manages 48+ agents with metadata
- **Agent Router** (`crates/core/src/agents/router.rs`): Intelligent routing based on task type
- **Hooks System** (`crates/core/src/hooks/`): 35+ hooks for automation
- **Configuration** (`crates/core/src/config.rs`): TOML-based configuration
- **Session** (`crates/core/src/session/`): Backend session management

### Agent Categories

- **Engineering & Development**: 8 agents (Architect, Code Generator, Code Reviewer, etc.)
- **Research & Analysis**: 6 agents (Codebase Analyzer, Research Analyzer, etc.)
- **Documentation & Planning**: 4 agents (Docbot Pro, Plan Implementation Reviewer, etc.)
- **Utilities & Tools**: 7 agents (Agent Creator, Skill Creator, Plugin Validator, etc.)
- **Creative & Entertainment**: 7 agents (Creative Writer, Pirate, Yoda, etc.)

## Building and Running

### Prerequisites

- Rust 1.70+ installed
- Python 3.8+ installed (for management scripts)
- Gemini CLI configured

### Build Process

```bash
# Build the release binary
just build

# Build with bundled PMAT
just build-bundled

# Build with full features
just build-full
```

### Running

```bash
# Run the server (MCP over stdio)
just run

# Run with bundled PMAT
just run-bundled

# Development mode with hot-reload
just dev
```

### Installation

```bash
# Install to ~/.local/bin
just install

# Install bundled version
just install-bundled
```

## Development Conventions

### Agent Management

- Agents are defined in `agents/` directory as Markdown files
- Agent metadata is stored in `agents/agents.json`
- Custom agents can be added to `custom/` directory

### Hook Development

- Hooks are implemented in `crates/core/src/hooks/`
- Each hook is a module with a creation function
- Hooks are registered in `crates/core/src/hooks/mod.rs`

### Configuration

- Configuration uses TOML format
- Supports multiple configuration locations
- Features intelligent routing with priority levels

### Testing

- Unit tests with Rust's built-in testing framework
- Integration tests for MCP protocol
- Run all tests with `just test`
- Code quality with `just clippy` and `just fmt`

## Commands

The extension provides the following CLI commands:

- `/system-agent` - List all available agents
- `/system-agent:info <agent_id>` - Get detailed information about an agent
- `/system-agent:switch <agent_id>` - Get instructions to switch to an agent
- `/system-agent:examples <agent_id>` - Show example prompts for an agent
- `/system-agent:new` - Interactive wizard to create a new agent

## Key Features

1. **48+ Specialized Agents**: Curated agents for various domains
2. **Intelligent Routing**: Agents selected based on task type and keywords
3. **35+ Hooks**: Advanced automation and context injection
4. **MCP Integration**: Model Context Protocol support
5. **Type Safety**: Full Rust type system for reliability
6. **Extensible Design**: Easy addition of custom agents and hooks

## Development Workflow

1. **Setup**: Run `just setup` to install development tools
2. **Development**: Use `just dev` for hot-reload mode
3. **Testing**: Run `just test` to execute all tests
4. **Quality**: Use `just clippy` and `just fmt` to maintain code quality
5. **Documentation**: Run `just doc` to generate Rust docs

## Configuration Structure

The system uses a hierarchical configuration:

```toml
[server]
host = "127.0.0.1"
port = 3000

[main_agent]
name = "gemini"

[[agents]]
id = "architect"
name = "Architect"
category = "engineering"

[[routing.rules]]
task_type = "code-generation"
preferred_agents = ["code-generator"]

[rate_limiting]
requests_per_minute = 60
requests_per_day = 2000
```

## Agent Capabilities

Agents are categorized by capabilities and can be selected based on:

- Task type (code generation, architecture, research, etc.)
- Keywords in the user prompt
- Priority levels
- Enabled/disabled status
- Rate limiting constraints

---

**Built with ❤️ using Rust and Tokio for high-performance AI agent orchestration.**
