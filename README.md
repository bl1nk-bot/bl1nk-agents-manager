# 🤖 BL1NK Agents Manager

![Version](https://img.shields.io/badge/version-0.2.0-blue.svg)
![License](https://img.shields.io/badge/license-MIT-green.svg)
![Status](https://img.shields.io/badge/status-active-success.svg)
![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)

> **Intelligent MCP/ACP Orchestrator with Bundled PMAT Support**

A high-performance Rust-based orchestrator that bridges MCP (Model Context Protocol) for Gemini CLI integration with ACP (Agent Client Protocol) for sub-agent communication.

---

## 📖 Table of Contents

- [Features](#-features)
- [Architecture](#-architecture)
- [Installation](#-installation)
- [Quick Start](#-quick-start)
- [Configuration](#-configuration)
- [Development](#-development)
- [Documentation](#-documentation)
- [License](#-license)

---

## ✨ Features

*   **Dual-Protocol Support**: Operates as both MCP Server and ACP Client simultaneously
*   **Intelligent Routing**: Task-aware agent selection with priority-based fallback
*   **Hook System**: Extensible hooks for PreToolUse, PostToolUse, PermissionRequest, and more
*   **Rate Limiting**: Per-agent quota tracking (requests/minute and requests/day)
*   **Background Tasks**: Async execution with task tracking
*   **Type Safety**: JSON Schema generation with compile-time validation
*   **PMAT Bundled**: Built-in context analysis with optional bundled PMAT

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Gemini MCP Proxy                          │
├─────────────────────────────────────────────────────────────┤
│  Layer 1: MCP Server (PMCP)                                │
│  └── TypedTools, JSON-RPC 2.0, stdio transport            │
├─────────────────────────────────────────────────────────────┤
│  Layer 2: Rate Limiting                                    │
│  └── Per-agent quota tracking, concurrent task management  │
├─────────────────────────────────────────────────────────────┤
│  Layer 3: Agent Management                                  │
│  ├── AgentRegistry     │ AgentRouter    │ AgentExecutor   │
│  └── HookAggregator                                           │
├─────────────────────────────────────────────────────────────┤
│  Layer 4: Protocol Adapters                                 │
│  └── ACP Client │ External Agents (CLI processes)           │
└─────────────────────────────────────────────────────────────┘
```

### Core Modules

| Module | Purpose |
|--------|---------|
| `src/mcp/` | MCP server implementation using PMCP SDK |
| `src/agents/` | Agent registry, routing, and execution |
| `src/hooks/` | Hook aggregator for event handling |
| `src/permissions/` | Permission management and rule parsing |
| `src/persistence/` | Task and state persistence |
| `src/system/` | System discovery and resource management |

---

## 📦 Installation

### Prerequisites

*   Rust 1.70+ ([Install](https://rustup.rs))
*   For bundled PMAT: `cargo build --features bundle-pmat`

### Build from Source

```bash
# Standard build
cargo build --release

# With bundled PMAT (recommended)
cargo build --release --features bundle-pmat

# Full features
cargo build --release --features bundle-pmat-full
```

### Quick Install

```bash
# Install to ~/.local/bin
make install-bundled
```

---

## 🚀 Quick Start

### 1. Configure Agents

Edit `config.toml` or create `~/.config/bl1nk-agents-manager/config.toml`:

```toml
[server]
host = "127.0.0.1"
port = 3000
max_concurrent_tasks = 5

[main_agent]
name = "gemini"
type = "gemini-cli"

[[agents]]
id = "qwen-coder"
name = "Qwen Coder"
type = "cli"
command = "qwencode"
rate_limit = { requests_per_minute = 60, requests_per_day = 2000 }
capabilities = ["code-generation", "refactoring"]

[routing]
rules = []
```

### 2. Run the Server

```bash
# Development mode
make dev

# Or release mode
make run-bundled
```

### 3. Connect to Gemini CLI

Add to your MCP config:

```json
{
  "mcpServers": {
    "bl1nk-proxy": {
      "command": "/path/to/bl1nk-agents-manager",
      "transport": "stdio"
    }
  }
}
```

---

## ⚙️ Configuration

### Agent Configuration

```toml
[[agents]]
id = "my-agent"
name = "My Agent"
type = "cli"                    # cli, gemini-extension, internal
command = "/path/to/agent"       # For cli type
args = ["--arg1", "value1"]
rate_limit = { requests_per_minute = 60, requests_per_day = 2000 }
capabilities = ["task-type"]
priority = 100                   # 0-255, higher = preferred
enabled = true
cost = 0                         # 0=free, <500=cheap, >=500=expensive
```

### Routing Rules

```toml
[routing]
tier = "user"  # default, user, admin (higher tier = higher priority)

[[routing.rules]]
task_type = "code-generation"
keywords = ["write", "code", "implement"]
preferred_agents = ["qwen-coder", "gpt-coder"]
priority = 500  # 0-999
enabled = true
```

---

## 🛠️ Development

### Project Structure

```
bl1nk-agents-manager/
├── src/
│   ├── main.rs              # Entry point
│   ├── config.rs             # TOML configuration
│   ├── rate_limit.rs         # Rate limiting
│   ├── agents/
│   │   ├── mod.rs           # Agent management
│   │   ├── register.rs      # Agent registry
│   │   ├── router.rs        # Smart routing
│   │   ├── extractor.rs      # Task execution
│   │   └── creator.rs        # Agent spec creation
│   ├── hooks/
│   │   ├── mod.rs           # Hook exports
│   │   └── hook_aggregator.rs # Hook execution & merging
│   ├── mcp/                 # MCP server (PMCP)
│   │   ├── mod.rs
│   │   └── protocol.rs
│   ├── permissions/          # Permission system
│   │   ├── permission_manager.rs
│   │   ├── rule_parser.rs
│   │   └── shell_semantics.rs
│   ├── persistence/          # Data persistence
│   └── system/              # System discovery
│       └── discovery.rs
├── commands/                 # CLI command definitions
│   └── agent/
├── scripts/                  # Python tools
│   └── test_integration.py  # Integration tests
├── docs/                     # Documentation
│   ├── ARCHITECTURE.md
│   ├── AGENT_GUIDE.md
│   ├── QUICKSTART.md
│   └── PROJECT_SUMMARY.md
├── Cargo.toml
├── Makefile
└── rustfmt.toml
```

### Available Make Commands

```bash
make help           # Show all commands
make build          # Standard release build
make build-bundled  # Build with bundled PMAT
make run            # Run standard build
make run-bundled    # Run with bundled PMAT
make dev            # Hot-reload development
make test           # Run all tests
make check          # Quick compilation check
make fmt            # Format code
make clippy         # Lint code
make lint           # Run all linters
make spellcheck     # Spell check
make all-check      # Run all checks
make clean          # Clean build artifacts
```

---

## 📚 Documentation

| Document | Description |
|----------|-------------|
| [ARCHITECTURE.md](docs/ARCHITECTURE.md) | System design and internals |
| [AGENT_GUIDE.md](docs/AGENT_GUIDE.md) | Creating ACP-compatible agents |
| [QUICKSTART.md](docs/QUICKSTART.md) | 5-minute getting started guide |
| [PROJECT_SUMMARY.md](docs/PROJECT_SUMMARY.md) | Project overview (Thai) |

---

## 🔧 Hook System

The system supports hooks for intercepting and modifying behavior:

### Available Events

- **PreToolUse**: Before a tool is used
- **PostToolUse**: After a tool is used
- **PostToolUseFailure**: After tool failure
- **Stop**: Stop execution
- **SubagentStop**: Stop a subagent
- **UserPromptSubmit**: User prompt submission
- **PermissionRequest**: Permission handling

### Example Hook Output

```json
{
  "decision": "allow",
  "reason": "Tool usage permitted",
  "continue_execution": true,
  "system_message": "Optional user message"
}
```

---

## 📄 License

MIT License - see [LICENSE](LICENSE) file for details.

---

**Built with ❤️ using Rust, Tokio, and PMCP**
