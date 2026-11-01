import subprocess
import json
import sys
import time
import os
import threading
from queue import Queue, Empty

BINARY_PATH = "target/release/bl1nk-agents-manager"

def enqueue_output(out, queue):
    for line in iter(out.readline, b''):
        queue.put(line)
    out.close()

def run_test():
    if not os.path.exists(BINARY_PATH):
        print(f"Error: Binary not found at {BINARY_PATH}")
        sys.exit(1)

    print(f"🚀 Starting Integration Test...")
    
    # Force logs to stderr
    env = os.environ.copy()
    env["RUST_LOG"] = "info" 
    
    proc = subprocess.Popen(
        [BINARY_PATH, "--config", "config_test.toml"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE, # Capture stderr for ready check
        env=env,
        bufsize=0 # Unbuffered
    )

    # Queue for reading stderr without blocking
    q_stderr = Queue()
    t_stderr = threading.Thread(target=enqueue_output, args=(proc.stderr, q_stderr))
    t_stderr.daemon = True
    t_stderr.start()

    try:
        # 1. Wait for Server Ready
        print("⏳ Waiting for server to start...")
        start_time = time.time()
        server_ready = False
        while time.time() - start_time < 5:
            try:
                line = q_stderr.get_nowait()
                line_str = line.decode().strip()
                print(f"[LOG] {line_str}")
                if "Starting MCP server" in line_str:
                    server_ready = True
                    break
            except Empty:
                time.sleep(0.1)
        
        if not server_ready:
            print("❌ Server failed to start within timeout.")
            return

        print("✅ Server is READY!")

        # 2. Test Initialize
        req = {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {"name": "test", "version": "1.0"}
            }
        }
        print(f"[CLIENT] Sending Init...")
        proc.stdin.write((json.dumps(req) + "\n").encode())
        proc.stdin.flush()

        resp_line = proc.stdout.readline()
        if not resp_line:
            print("❌ No response from server.")
            return
        
        resp = json.loads(resp_line.decode())
        print(f"[SERVER] Init Response: {resp}")
        
        if "result" in resp:
            print("✅ Handshake Success!")
        else:
            print("❌ Handshake Failed!")

        # 3. Test Delegate Task
        req = {
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": {
                "name": "delegate_task",
                "arguments": {
                    "task_type": "code-generation",
                    "prompt": "test prompt",
                    "background": False
                }
            }
        }
        print(f"[CLIENT] Sending Task...")
        proc.stdin.write((json.dumps(req) + "\n").encode())
        proc.stdin.flush()

        resp_line = proc.stdout.readline()
        resp = json.loads(resp_line.decode())
        print(f"[SERVER] Task Response: {resp}")

        if "result" in resp:
             print("✅ Task Delegation Success!")
        else:
             print("❌ Task Delegation Failed!")

    except Exception as e:
        print(f"❌ Exception: {e}")
    finally:
        print("🧹 Cleaning up...")
        proc.terminate()
        proc.wait()

if __name__ == "__main__":
    run_test()