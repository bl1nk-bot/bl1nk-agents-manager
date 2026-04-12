import os
import re
import json

agents_dir = "agents/"
files = [f for f in os.listdir(agents_dir) if f.endswith(".md")]

# รายชื่อเอเจนต์ระดับ Elite (Built-in)
ELITE_SQUAD = {
    "orchestrator": {"mode": "all", "type": "general", "name": "Team Orchestrator"},
    "code-architect": {"mode": "primary", "type": "plan", "name": "System Architect"},
    "fullstack-dev": {"mode": "all", "type": "code", "name": "Fullstack Expert"},
    "code-generator": {"mode": "primary", "type": "code", "name": "Code Specialist"},
    "docbot-pro": {"mode": "primary", "type": "docs", "name": "Documentation Lead"},
    "codebase-analyzer": {"mode": "subagent", "type": "analysis", "name": "Structure Analyzer"},
    "ui-engineer": {"mode": "subagent", "type": "create", "name": "UI Specialist"},
    "github-issue-fixer": {"mode": "subagent", "type": "code", "name": "Issue Resolver"}
}

print(f"🧹 Reforming Agent Library into Elite Squad...")

for f in files:
    agent_id = f.replace(".md", "")
    path = os.path.join(agents_dir, f)
    
    with open(path, 'r', encoding='utf-8') as file:
        content = file.read()
    
    if agent_id in ELITE_SQUAD:
        # --- ยกระดับเป็น ELITE AGENT ---
        meta = ELITE_SQUAD[agent_id]
        
        # ปรับปรุง Frontmatter ให้มีความเป็นอำนาจ (Authority)
        new_yaml = [
            "---",
            f"id: {agent_id}",
            f"name: {meta['name']}",
            f"description: สุดยอดผู้เชี่ยวชาญด้าน {meta['name']} (Built-in Elite) ทำหน้าที่เป็นเสาหลักในงานประเภท {meta['type']}",
            f"mode: {meta['mode']}",
            f"type: {meta['type']}",
            "model: opus",
            "color: \"#FFD700\"", # สีทองสำหรับ Elite
            "tool:",
            "  bash: true",
            "  write: true",
            "  skill: true",
            "  ask: true",
            "permission: 900",
            "permission_policy:",
            "  hierarchy: [admin, user, workspace]",
            "  decision_rules:",
            "    - toolName: \"bash\"",
            "      commandPrefix: \"cargo \"",
            "      decision: \"allow\"",
            "      priority: 100",
            "      reason: \"Allow safe development commands\"",
            "    - toolName: \"*\"",
            "      decision: \"ask_user\"",
            "      priority: 0",
            "      reason: \"Default to safe confirmation\"",
            "  weight:",
            "    mode: 0.3",
            "    type: 0.3",
            "    tool: 0.2",
            "    evidence: 0.2",
            f"capabilities: [{agent_id}]",
            "---"
        ]
        print(f"  🌟 PROMOTED: {f:30} -> AGENT ({meta['mode']})")
    else:
        # --- ลดขั้นเป็น SKILL ---
        new_yaml = [
            "---",
            f"id: {agent_id}",
            f"name: [Skill] {agent_id.replace('-', ' ').title()}",
            f"description: ชุดทักษะและความรู้ด้าน {agent_id} สำหรับให้เอเจนต์หลักเรียกใช้งานอ้างอิง",
            "mode: subagent",
            "type: general",
            "model: sonnet",
            "tool:",
            "  bash: false",
            "  write: false",
            "  skill: true",
            "  ask: false",
            "permission: 100",
            "permission_policy:",
            "  hierarchy: [default]",
            "  decision_rules: [{toolName: \"*\", decision: \"deny\"}]",
            f"capabilities: [{agent_id}]",
            "---"
        ]
        print(f"  📦 DEMOTED:  {f:30} -> SKILL")

    # แยก Body เดิม (Prompt)
    parts = re.split(r'^---$', content, flags=re.MULTILINE)
    body = parts[2] if len(parts) >= 3 else content
    
    with open(path, 'w', encoding='utf-8') as file:
        file.write("\n".join(new_yaml) + "\n" + body)

print("\n🚀 Elite Squad Reform Complete.")
