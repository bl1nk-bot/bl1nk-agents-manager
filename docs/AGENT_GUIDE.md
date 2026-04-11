# Creating ACP-Compatible Agents

This guide shows how to create agents that work with Gemini MCP Proxy.

## Agent Requirements

### 1. JSON-RPC 2.0 Protocol

Agents must communicate via JSON-RPC 2.0 over stdin/stdout:

**Request Format:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "execute_task",
    "arguments": {
      "prompt": "User's instruction",
      "context": {}
    }
  }
}
```

**Response Format:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": "Agent's response here"
}
```

**Error Format:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32000,
    "message": "Error description"
  }
}
```

### 2. stdin/stdout Communication

- Read requests from **stdin** (one JSON per line)
- Write responses to **stdout** (one JSON per line)
- Write logs to **stderr** (optional)

### 3. Exit Cleanly

- Exit with code 0 on success
- Exit with non-zero on error
- Flush stdout before exit

## Example Agents

### Python Agent (Minimal)

```python
#!/usr/bin/env python3
import json
import sys

def handle_request(request):
    """Process JSON-RPC request"""
    method = request.get("method")
    params = request.get("params", {})
    
    if method == "tools/call":
        tool_name = params.get("name")
        args = params.get("arguments", {})
        
        if tool_name == "execute_task":
            prompt = args.get("prompt", "")
            # Do work here
            result = f"Processed: {prompt}"
            
            return {
                "jsonrpc": "2.0",
                "id": request.get("id"),
                "result": result
            }
    
    return {
        "jsonrpc": "2.0",
        "id": request.get("id"),
        "error": {
            "code": -32601,
            "message": "Method not found"
        }
    }

def main():
    for line in sys.stdin:
        try:
            request = json.loads(line)
            response = handle_request(request)
            print(json.dumps(response), flush=True)
        except Exception as e:
            error_response = {
                "jsonrpc": "2.0",
                "id": None,
                "error": {
                    "code": -32603,
                    "message": str(e)
                }
            }
            print(json.dumps(error_response), flush=True)
            sys.exit(1)

if __name__ == "__main__":
    main()
```

### Rust Agent (Using PMCP)

```rust
use pmcp::{ServerBuilder, TypedTool, RequestHandlerExtra};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Debug, Deserialize, JsonSchema)]
struct TaskArgs {
    prompt: String,
    context: Option<serde_json::Value>,
}

async fn execute_task(
    args: TaskArgs,
    _extra: RequestHandlerExtra
) -> pmcp::Result<String> {
    // Process the task
    Ok(format!("Processed: {}", args.prompt))
}

#[tokio::main]
async fn main() -> pmcp::Result<()> {
    let server = ServerBuilder::new()
        .name("my-agent")
        .version("1.0.0")
        .tool("execute_task", TypedTool::new(
            "execute_task",
            |args, extra| Box::pin(execute_task(args, extra))
        ))
        .build()?;
    
    server.run_stdio().await?;
    Ok(())
}
```

### Node.js Agent

```javascript
#!/usr/bin/env node
const readline = require('readline');

const rl = readline.createInterface({
  input: process.stdin,
  output: process.stdout,
  terminal: false
});

rl.on('line', (line) => {
  try {
    const request = JSON.parse(line);
    const response = handleRequest(request);
    console.log(JSON.stringify(response));
  } catch (error) {
    console.error(JSON.stringify({
      jsonrpc: "2.0",
      id: null,
      error: {
        code: -32603,
        message: error.message
      }
    }));
    process.exit(1);
  }
});

function handleRequest(request) {
  const { method, params, id } = request;
  
  if (method === 'tools/call' && params.name === 'execute_task') {
    const { prompt } = params.arguments;
    return {
      jsonrpc: "2.0",
      id,
      result: `Processed: ${prompt}`
    };
  }
  
  return {
    jsonrpc: "2.0",
    id,
    error: {
      code: -32601,
      message: "Method not found"
    }
  };
}
```

## Testing Your Agent

### Manual Test

```bash
# Test stdin/stdout communication
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"execute_task","arguments":{"prompt":"test"}}}' | ./my-agent
```

Expected output:
```json
{"jsonrpc":"2.0","id":1,"result":"Processed: test"}
```

### Integration with Proxy

1. Add agent to config:
```toml
[[agents]]
id = "my-agent"
name = "My Custom Agent"
type = "cli"
command = "/path/to/my-agent"
rate_limit = { requests_per_minute = 60, requests_per_day = 2000 }
capabilities = ["custom-task"]
priority = 1
```

2. Add routing rule:
```toml
[[routing.rules]]
task_type = "custom-task"
keywords = ["custom"]
preferred_agents = ["my-agent"]
```

3. Test delegation:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "delegate_task",
    "arguments": {
      "task_type": "custom-task",
      "prompt": "Do custom work",
      "agent_id": "my-agent"
    }
  }
}
```

## Best Practices

### 1. Error Handling

Always catch exceptions and return proper JSON-RPC errors:

```python
try:
    result = process_task(prompt)
    return {"jsonrpc": "2.0", "id": req_id, "result": result}
except ValueError as e:
    return {
        "jsonrpc": "2.0",
        "id": req_id,
        "error": {"code": -32602, "message": str(e)}
    }
```

### 2. Logging

Write logs to stderr, not stdout:

```python
import sys

def log(message):
    print(f"[Agent] {message}", file=sys.stderr, flush=True)

log("Processing request...")
```

### 3. Flush Output

Always flush stdout after writing response:

```python
print(json.dumps(response), flush=True)
```

```javascript
console.log(JSON.stringify(response)); // Automatically flushes
```

```rust
println!("{}", serde_json::to_string(&response)?);
// stdout auto-flushes on newline
```

### 4. Timeout Handling

Handle long-running tasks:

```python
import signal

def timeout_handler(signum, frame):
    raise TimeoutError("Task timed out")

signal.signal(signal.SIGALRM, timeout_handler)
signal.alarm(300)  # 5 minute timeout

try:
    result = long_running_task()
    signal.alarm(0)  # Cancel alarm
except TimeoutError:
    return error_response("Task timed out after 5 minutes")
```

### 5. Rate Limit Reporting

Include rate limit info in responses (optional):

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": "...",
  "meta": {
    "rate_limit": {
      "remaining_today": 1950,
      "remaining_minute": 55
    }
  }
}
```

## Common Issues

### Issue: Agent hangs

**Cause**: Not flushing stdout
**Fix**: Add `flush=True` or call `sys.stdout.flush()`

### Issue: JSON parse error

**Cause**: Multiple JSONs on one line
**Fix**: One JSON per line, terminated with `\n`

### Issue: Agent exits immediately

**Cause**: Reading all stdin at once
**Fix**: Read line-by-line in a loop

### Issue: Garbled output

**Cause**: Mixing stdout/stderr
**Fix**: Only write JSON to stdout, logs to stderr

## Advanced Features

### Streaming Responses

For long-running tasks, send progress updates:

```python
def stream_progress(task_id):
    for i in range(100):
        progress = {
            "jsonrpc": "2.0",
            "method": "progress",
            "params": {
                "task_id": task_id,
                "percent": i
            }
        }
        print(json.dumps(progress), flush=True)
        time.sleep(0.1)
```

### Context Passing

Use context for state:

```python
def handle_with_context(request):
    context = request["params"]["arguments"].get("context", {})
    previous_result = context.get("previous_result")
    
    # Use previous result
    new_result = process(previous_result)
    
    return {
        "result": new_result,
        "context": {
            "previous_result": new_result
        }
    }
```

## Agent Checklist

Before deploying:

- [ ] Implements JSON-RPC 2.0 protocol
- [ ] Reads from stdin, writes to stdout
- [ ] Flushes output after each response
- [ ] Handles errors gracefully
- [ ] Logs to stderr (not stdout)
- [ ] Exits cleanly (code 0 on success)
- [ ] Tested manually with echo/pipe
- [ ] Added to proxy config
- [ ] Tested via proxy delegation

---

**Need help?** Check examples in `/examples` directory or open an issue.
## Hooks and Event Handling

The system supports hooks that can intercept and modify behavior at various points. As an agent developer, you can write custom hooks to extend functionality.

### Available Hook Events

- **PreToolUse**: Before a tool is used
- **PostToolUse**: After a tool is used
- **PostToolUseFailure**: After tool failure
- **Stop**: Can stop execution
- **SubagentStop**: Stops a subagent
- **UserPromptSubmit**: When user submits a prompt
- **PermissionRequest**: For permission handling

### Hook Output Format

Hooks should return JSON responses with the following structure:

```json
{
  "decision": "allow|block|deny",
  "reason": "Explanation of the decision",
  "continue_execution": true|false,
  "stop_reason": "Why execution stopped",
  "suppress_output": true|false,
  "system_message": "Message to display to user",
  "hook_specific_output": {
    "additionalContext": "Extra context for merging",
    "decision": { /* For PermissionRequest hooks */ }
  }
}
```

### Hook Merging Logic

Multiple hooks can be registered for the same event. Their outputs are merged using event-specific logic:

- **OR Logic**: For tool use events, a "block" or "deny" decision from any hook wins
- **PermissionRequest**: Specialized merging for decision objects and permissions
- **Simple Merge**: For other events, latest values win with context concatenation

### Example Hook

```python
#!/usr/bin/env python3
import json
import sys

def handle_hook(event_name, payload):
    """Process a hook event"""
    if event_name == "PreToolUse":
        tool_name = payload.get("tool_name")
        if tool_name == "write_file" and "sensitive" in payload.get("file_path", ""):
            return {
                "decision": "deny",
                "reason": "Cannot write to sensitive files",
                "continue_execution": False,
                "stop_reason": "Security policy violation"
            }
    return {
        "decision": "allow",
        "continue_execution": True
    }

def main():
    for line in sys.stdin:
        try:
            request = json.loads(line)
            response = handle_hook(
                request.get("method"),
                request.get("params", {})
            )
            print(json.dumps(response), flush=True)
        except Exception as e:
            error_response = {
                "decision": "deny",
                "reason": f"Error processing hook: {str(e)}",
                "continue_execution": False
            }
            print(json.dumps(error_response), flush=True)
            sys.exit(1)

if __name__ == "__main__":
    main()
```

### Registering Hooks

Hooks are registered in the system configuration. Each hook specifies:
- Event name to listen for
- Command to execute
- Priority/order (optional)

When the event occurs, all registered hooks are executed, and their results are aggregated.

### Best Practices

1. **Return early**: Return allow if no special handling needed
2. **Be specific**: Only block when absolutely necessary
3. **Provide clear reasons**: Help users understand decisions
4. **Handle errors gracefully**: Never crash the hook process
5. **Keep it fast**: Hooks should execute quickly to avoid latency

### Common Use Cases

- **Security policies**: Block unauthorized actions
- **Audit logging**: Record all operations
- **Context enrichment**: Add metadata to tool calls
- **Dynamic routing**: Modify agent selection based on conditions

---
