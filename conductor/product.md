# Initial Concept

Intelligent MCP/ACP orchestrator with bundled PMAT support

---

# Product Guide

## Vision
Transform the Gemini CLI into a specialized multi-persona AI workforce by orchestrating MCP (Model Context Protocol) and ACP (Agent Communication Protocol) agents with intelligent routing, rate limiting, and task delegation.

## Core Problem
Single-persona AI assistants struggle with domain-specific tasks. Users need context-aware routing to specialized AI agents that follow strict behavioral rules, output formats, and domain-specific best practices.

## Solution
BL1NK Agents Manager acts as an intelligent orchestrator layer between users and AI agents, providing:
- **Agent Management:** Library of specialized AI agents (40+ personas)
- **Smart Routing:** Keyword-based and capability-based agent selection
- **Task Delegation:** Interactive CLI with proposal/review workflow
- **Rate Limiting:** Per-agent request throttling (RPM/RPD)
- **Persistence:** Session and task history tracking
- **Hooks:** Extensible pre/post task execution hooks

## Target Users
1. **Developers** using Gemini CLI who need specialized AI personas for different tasks
2. **Teams** managing shared AI agent libraries with consistent configurations
3. **Power Users** requiring advanced routing, rate limiting, and audit capabilities

## Key Features
### MVP (Current)
- [x] Agent registry with metadata validation
- [x] TOML-based configuration with multi-path resolution
- [x] MCP protocol server (stdio transport)
- [x] CLI delegation with interactive agent selection
- [x] Routing rules with priority tiers (Default/User/Admin)
- [x] Rate limiting per agent (RPM/RPD)
- [x] System discovery engine
- [x] Tracing-based logging with JSON output

### Planned
- [ ] HTTP/WebSocket transport for remote agents
- [ ] Agent health monitoring and auto-failover
- [ ] Usage analytics dashboard
- [ ] Plugin system for custom routing strategies
- [ ] Multi-tenant agent isolation
- [ ] ACP protocol support
- [ ] Web UI for agent management

## Architecture
```
┌─────────────────────────────────────────────────┐
│                  CLI Interface                   │
│         (clap: delegate, status, list)          │
└──────────────────┬──────────────────────────────┘
                   │
┌──────────────────▼──────────────────────────────┐
│              Orchestrator Core                   │
│  ┌────────────┐  ┌──────────┐  ┌────────────┐  │
│  │   Router   │→ │ Scheduler │→ │  Executor  │  │
│  └────────────┘  └──────────┘  └────────────┘  │
│         │              │              │         │
│  ┌──────▼──────┐ ┌────▼─────┐ ┌──────▼──────┐  │
│  │Rule Engine  │ │Rate Limit│ │  Hook System │  │
│  └─────────────┘ └──────────┘ └─────────────┘  │
└──────────────────┬──────────────────────────────┘
                   │
┌──────────────────▼──────────────────────────────┐
│               MCP Protocol Layer                 │
│         (stdio JSON-RPC 2.0 server)             │
└──────────────────┬──────────────────────────────┘
                   │
┌──────────────────▼──────────────────────────────┐
│              Agent Processes                     │
│  (CLI extensions, HTTP services, etc.)          │
└─────────────────────────────────────────────────┘
```

## Non-Functional Requirements
- **Performance:** <100ms routing decision latency
- **Reliability:** Graceful degradation when agents fail
- **Security:** Permission sandboxing for shell execution
- **Scalability:** Support 100+ concurrent agent sessions
- **Observability:** Structured JSON logs with correlation IDs

## Success Metrics
- Agent routing accuracy >95%
- Task completion rate >90%
- Average response latency <2s
- Zero configuration-related crashes on startup
