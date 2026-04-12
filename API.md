# API Reference

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
## Overview

Bl1nk Agents Manager provides a **Model Context Protocol (MCP)** interface for agent orchestration. The MCP server runs over stdio, allowing seamless integration with Claude CLI and other MCP-compatible clients.

### Key Capabilities

- **Agent Delegation**: Delegate tasks to specialized agents based on task type
- **Agent Status**: Query available agents and running task status
- **Session Management**: Manage backend sessions with multiple model providers
- **Filesystem Operations**: Read, write, and manage files
- **Search**: Search conversation history and projects

### Transport

The MCP server communicates over **stdio** (standard input/output) using JSON-RPC 2.0 protocol.

### Protocol

All communication uses **JSON-RPC 2.0** format:

```json
// Request
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "delegate_task",
    "arguments": {
      "task_type": "code-generation",
      "prompt": "Write a function to calculate fibonacci",
      "background": false
    }
  }
}

// Response
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "task_id": "task-123",
    "agent_id": "code-generator",
    "status": "completed",
    "result": "..."
  }
}
```

---

## MCP Tools

### 1. delegate_task

Delegates a task to an appropriate sub-agent based on task type.

**Parameters:**

```json
{
  "task_type": "string",      // Type of task (e.g., "code-generation", "research")
  "prompt": "string",         // Task instruction
  "agent_id": "string|null",  // Optional specific agent ID
  "background": boolean,      // Run as background task
  "context": object|null       // Additional context
}
```

**Response:**

```json
{
  "task_id": "string",
  "agent_id": "string",
  "status": "string",
  "result": "string|null"
}
```

**Example:**

```bash
# Using MCP client
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"delegate_task","arguments":{"task_type":"code-generation","prompt":"Write a hello world in Rust","background":false}}}' | ./target/release/bl1nk-server
```

---

### 2. agent_status

Queries the status of agents and running tasks.

**Parameters:**

```json
{
  "task_id": "string|null"  // Optional specific task ID
}
```

**Response:**

```json
{
  "active_tasks": 0,
  "available_agents": ["architect", "code-generator", "pirate", ...],
  "task_info": {
    "task_id": "string",
    "status": "string"
  }
}
```

---

## Session Management API

### Initialize Session

Creates a new backend session.

**Parameters:**

```json
{
  "session_id": "string",
  "working_directory": "string",
  "model": "string",
  "backend_config": "object|null",
  "gemini_auth": "object|null",
  "llxprt_config": "object|null"
}
```

### Send Message

Sends a message to the active agent session.

**Parameters:**

```json
{
  "session_id": "string",
  "message": "string",
  "conversation_history": "string"
}
```

### Process Status

Retrieves status of all running processes.

**Response:**

```json
[
  {
    "conversation_id": "string",
    "is_alive": boolean
  }
]
```

---

## Filesystem Operations

### Read File Content

Reads the content of a specified file.

**Parameters:**

```json
{
  "path": "string",
  "follow_symlinks": true
}
```

**Response:**

```json
{
  "path": "string",
  "content": "string",
  "size": 1024,
  "modified": "2026-02-06T12:00:00Z"
}
```

### Write File Content

Writes content to a specified file.

**Parameters:**

```json
{
  "path": "string",
  "content": "string"
}
```

### List Directory

Lists contents of a directory.

**Parameters:**

```json
{
  "path": "string"
}
```

**Response:**

```json
[
  {
    "name": "string",
    "path": "string",
    "is_dir": boolean,
    "size": 1024
  }
]
```

### Get Canonical Path

Gets the absolute canonical path for a given path.

**Parameters:**

```json
{
  "path": "string"
}
```

**Response:**

```json
{
  "canonical_path": "/absolute/path/to/relative/path"
}
```

### Get Git Info

Retrieves git repository information.

**Parameters:**

```json
{
  "path": "string"
}
```

**Response:**

```json
{
  "branch": "main",
  "commit": "abc123def",
  "tags": ["v1.0.0"]
}
```

---

## Search API

### Search Chats

Searches through conversation history.

**Parameters:**

```json
{
  "q": "search query",
  "filters": {
    "date_from": "2026-01-01",
    "date_to": "2026-02-06",
    "agents": ["architect", "code-generator"]
  }
}
```

**Response:**

```json
{
  "results": [
    {
      "id": "chat-123",
      "title": "Search result title",
      "snippet": "Relevant excerpt...",
      "date": "2026-02-01",
      "agents": ["architect"]
    }
  ],
  "total": 10
}
```

### Get Detailed Conversation

Retrieves full conversation details.

**Parameters:**

```json
{
  "id": "conversation-id"
}
```

**Response:**

```json
{
  "id": "conversation-id",
  "title": "Conversation Title",
  "date": "2026-02-01T12:00:00Z",
  "messages": [
    {
      "role": "user",
      "content": "How do I build a web server?",
      "timestamp": "2026-02-01T12:00:01Z"
    },
    {
      "role": "assistant",
      "content": "Here's how to build a web server in Rust...",
      "timestamp": "2026-02-01T12:00:02Z",
      "agent": "code-generator"
    }
  ]
}
```

### Export Conversation

Exports conversation history in various formats.

**Parameters:**

```json
{
  "id": "conversation-id",
  "format": "json|markdown|text"
}
```

**Response:**

```json
{
  "format": "json",
  "content": "..."
}
```

---

## Project API

### List Projects

Lists all projects with pagination.

**Parameters:**

```json
{
  "limit": 20,
  "offset": 0
}
```

**Response:**

```json
{
  "projects": [
    {
      "id": "project-123",
      "name": "My Project",
      "path": "/path/to/project",
      "last_modified": "2026-02-06",
      "language": "rust"
    }
  ],
  "total": 50
}
```

### Get Enriched Project

Gets detailed project information including discussions.

**Parameters:**

```json
{
  "sha": "git-commit-sha",
  "path": "/path/to/project"
}
```

**Response:**

```json
{
  "project": {
    "id": "project-123",
    "name": "My Project",
    "path": "/path/to/project",
    "files": 100,
    "languages": ["rust", "typescript"]
  },
  "discussions": [
    {
      "id": "discussion-1",
      "title": "Architecture decision",
      "date": "2026-02-01"
    }
  ]
}
```

---

## Error Handling

All errors follow JSON-RPC 2.0 error format:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32000,
    "message": "Error description",
    "data": {
      "details": "Additional error information"
    }
  }
}
```

### Common Error Codes

| Code | Meaning |
|------|---------|
| -32600 | Invalid Request |
| -32601 | Method Not Found |
| -32602 | Invalid Parameters |
| -32603 | Internal Error |
| -32000 | Application Error |
| -32001 | Rate Limit Exceeded |
| -32002 | Agent Not Found |
| -32003 | Session Not Found |

---

## Rate Limiting

The system implements rate limiting per agent:

- **Requests per minute**: 60 (default)
- **Requests per day**: 2000 (default)

Rate limit status is included in responses:

```json
{
  "result": "...",
  "rate_limit": {
    "remaining_minute": 45,
    "remaining_day": 1950
  }
}
```

---

## Configuration

### TOML Configuration

```toml
[server]
host = "127.0.0.1"
port = 3000

[main_agent]
name = "gemini"
type = "gemini-cli"

[[agents]]
id = "architect"
name = "Architect"
type = "builtin"

[[routing.rules]]
task_type = "code-generation"
preferred_agents = ["code-generator"]

[rate_limiting]
requests_per_minute = 60
requests_per_day = 2000
```

---

## Integration Examples

### Integration with Claude CLI

1. Add to Claude CLI config:

```json
{
  "mcpServers": {
    "bl1nk": {
      "command": "/path/to/bl1nk-server",
      "transport": "stdio"
    }
  }
}
```

1. Use from Claude:

```
User: "Delegate this task to the architect agent"
Claude: [calls delegate_task via MCP]
```

### Using with Custom MCP Client

```python
import subprocess
import json

def send_mcp_request(method, params):
    request = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": method,
        "params": params
    }
    
    proc = subprocess.Popen(
        ["./target/release/bl1nk-server"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        text=True
    )
    
    stdout, _ = proc.communicate(json.dumps(request))
    return json.loads(stdout)

# Delegate a task
response = send_mcp_request("tools/call", {
    "name": "delegate_task",
    "arguments": {
        "task_type": "code-generation",
        "prompt": "Write a REST API in Rust",
        "background": false
    }
})
```

---

**Last updated**: 2026-02-06
