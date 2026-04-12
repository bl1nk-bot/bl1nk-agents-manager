# Quick Start Guide

## 🚀 Get Started in 5 Minutes

### Step 1: Prerequisites

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Verify installation
rustc --version
cargo --version
```

### Step 2: Project Setup

```bash
# Navigate to project directory
cd bl1nk-agents-manager

# Install development tools (optional)
make setup

# Build the project
make build-bundled
```

### Step 3: Configuration

```bash
# Create config directory
mkdir -p ~/.config/bl1nk-agents-manager

<<<<<<< HEAD
# Copy and customize config from the project's .config/ directory
cp .config/config.example.toml ~/.config/bl1nk-agents-manager/config.toml
=======
# Copy and customize config
cp config.example.toml ~/.config/bl1nk-agents-manager/config.toml
>>>>>>> origin/dev

# Edit config to add your agents
nano ~/.config/bl1nk-agents-manager/config.toml
```

**Minimal working config:**

```toml
[server]
host = "127.0.0.1"
port = 3000
max_concurrent_tasks = 5

[main_agent]
name = "gemini"
type = "gemini-cli"

[[agents]]
id = "test-agent"
name = "Test Agent"
type = "cli"
command = "echo"
args = ["Hello from agent"]
rate_limit = { requests_per_minute = 60, requests_per_day = 2000 }
capabilities = ["test"]
priority = 1

[routing]
rules = []

[rate_limiting]
strategy = "round-robin"
track_usage = true
usage_db_path = "~/.config/gemini-mcp-proxy/usage.db"

[logging]
level = "info"
output = "stdout"
```

### Step 4: Run the Server

```bash
# Run with bundled PMAT (recommended)
make run-bundled

# Or in development mode
make dev

# Or with debug logging
RUST_LOG=debug cargo run --release --features bundle-pmat
```

Expected output:
```
2026-04-11T00:00:00Z INFO  Starting BL1NK Agents Manager
2026-04-11T00:00:00Z INFO  Loaded configuration with 3 agents
2026-04-11T00:00:00Z INFO  Starting MCP server on stdio
```

### Step 5: Test with MCP Client

Create a test script `test-client.sh`:

```bash
#!/bin/bash

# Send MCP initialize request
cat << 'EOF' | ./target/release/bl1nk-agents-manager
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0.0"}}}
EOF
```

Run test:
```bash
chmod +x test-client.sh
./test-client.sh
```

Run test:
```bash
chmod +x test-client.sh
./test-client.sh
```

## Common Use Cases

### Use Case 1: Delegate Task to Agent

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "delegate_task",
  "params": {
    "agent_id": "qwen-coder",
    "prompt": "Hello, agent!",
    "background": false
  }
}
```

### Use Case 2: Check Agent Status

```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "agent_status",
    "arguments": {}
  }
}
```

### Use Case 3: Background Task

```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "tools/call",
  "params": {
    "name": "delegate_task",
    "arguments": {
      "task_type": "background-task",
      "prompt": "npm install dependencies",
      "background": true
    }
  }
}
```

## Integration with Gemini CLI

### Option 1: Direct stdio

```bash
# Run as MCP server
bl1nk-agents-manager

# Gemini CLI connects via stdio
```

### Option 2: MCP Config

Add to your MCP client configuration:

```json
{
  "mcpServers": {
    "bl1nk-proxy": {
      "command": "/path/to/bl1nk-agents-manager",
      "args": [],
      "transport": "stdio"
    }
  }
}
```

## Troubleshooting

### Error: "cargo: not found"

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### Error: "No config file found"

```bash
# Create config
mkdir -p ~/.config/bl1nk-agents-manager
<<<<<<< HEAD
cp .config/config.example.toml ~/.config/bl1nk-agents-manager/config.toml
=======
cp config.example.toml ~/.config/bl1nk-agents-manager/config.toml
>>>>>>> origin/dev
```

### Error: "Agent command not found"

Check:
1. Command exists: `which qwencode`
2. Command is executable: `chmod +x /path/to/qwencode`
3. PATH is correct: `echo $PATH`

### Error: "Rate limit exceeded"

Reset usage database:
```bash
rm ~/.config/bl1nk-agents-manager/usage.db
```

## Next Steps

1. **Read Architecture**: See `ARCHITECTURE.md` for design details
2. **Add Real Agents**: Configure actual CLI agents (qwencode, codex, etc.)
3. **Customize Routing**: Add routing rules in config
4. **Deploy**: Use `make install` to install system-wide

## Development Workflow

```bash
# 1. Make changes to src/
# 2. Format code
make fmt

# 3. Check for errors
make check

# 4. Run tests
make test

# 5. Run clippy
make clippy

# 6. Build release
make build
```

## Resources

- **README.md** - Comprehensive documentation
- **ARCHITECTURE.md** - System design and internals
- **.config/config.example.toml** - Full config example
- **Makefile** - Development commands

---

**Ready to go!** 🎉

For questions or issues, check the README or open an issue.