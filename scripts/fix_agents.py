import os
import re

agents_dir = "agents/"
files = [f for f in os.listdir(agents_dir) if f.endswith(".md")]

print(f"🛠️ Re-Normalizing {len(files)} agents with Gemini CLI Policy Hierarchy...")

def fix_content(filename, content):
    match = re.match(r'^---\s*\n(.*?)\n---\s*\n(.*)$', content, re.DOTALL)
    if not match: return content
    
    yaml_text = match.group(1)
    body = match.group(2)
    
    def get_val(key, default=""):
        m = re.search(fr'^{key}:\s*(.*)$', yaml_text, re.MULTILINE)
        return m.group(1).strip().strip('"').strip("'") if m else default

    name = get_val("name", filename.replace(".md", "").title())
    name = re.sub(r'[^a-zA-Z0-9ก-๙\s\-&]', '', name)
    agent_id = filename.replace(".md", "")
    
    # กำหนด Tier ตามชื่อ/บทบาท
    hierarchy = ["workspace", "extension", "default"]
    if "orchestrator" in agent_id or "architect" in agent_id:
        hierarchy = ["admin", "user", "workspace"]

    new_yaml = [
        "---",
        f"id: {agent_id}",
        f"name: {name}",
        f"description: ผู้เชี่ยวชาญด้าน {name} ใช้เมื่อต้องการจัดการงานที่เกี่ยวข้องเพื่อให้ประสิทธิภาพสูงสุด",
        "mode: all",
        "type: general",
        "model: sonnet",
        "color: \"#38A3EE\"",
        "tool:",
        "  bash: false",
        "  write: false",
        "  skill: true",
        "  ask: false",
        "permission: 500",
        "permission_policy:",
        f"  hierarchy: {json.dumps(hierarchy)}",
        "  decision_rules:",
        "    - toolName: \"*\"",
        "      decision: \"ask_user\"",
        "  weight:",
        "    mode: 0.3",
        "    type: 0.3",
        "    tool: 0.2",
        "    evidence: 0.2",
        f"capabilities: [{agent_id}]",
        "---"
    ]
    
    return "\n".join(new_yaml) + "\n" + body

import json
for f in files:
    path = os.path.join(agents_dir, f)
    with open(path, 'r', encoding='utf-8') as file:
        content = file.read()
    new_content = fix_content(f, content)
    with open(path, 'w', encoding='utf-8') as file:
        file.write(new_content)
    print(f"✅ Re-Normalized: {f}")

print("\n🚀 Policy Engine Alignment Complete.")
