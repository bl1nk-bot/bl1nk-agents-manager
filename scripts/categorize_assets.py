import os
import re

agents_dir = "agents/"
files = [f for f in os.listdir(agents_dir) if f.endswith(".md")]

# นิยามของคำที่บ่งบอกว่าเป็น Skill (เอกสาร/ทักษะ) ไม่ใช่ตัวบุคคล
SKILL_KEYWORDS = [
    "guidelines", "patterns", "system", "workflow", "diagrams", 
    "locator", "analyzer", "template", "verifier", "researcher",
    "management", "reflector"
]

def categorize(filename):
    name_lower = filename.lower()
    if any(k in name_lower for k in SKILL_KEYWORDS):
        return "skill"
    return "agent"

print(f"🧐 Categorizing {len(files)} assets...")

for f in files:
    kind = categorize(f)
    path = os.path.join(agents_dir, f)
    
    with open(path, 'r', encoding='utf-8') as file:
        content = file.read()
    
    # 1. ปรับปรุง Frontmatter ให้ระบุ kind และจำกัดสิทธิ์หากเป็น Skill
    if kind == "skill":
        # ปรับ tool ให้เป็น false ทั้งหมด และลด permission
        content = re.sub(r'bash:\s*true', 'bash: false', content)
        content = re.sub(r'write:\s*true', 'write: false', content)
        content = re.sub(r'ask:\s*true', 'ask: false', content)
        content = re.sub(r'permission:\s*\d+', 'permission: 100', content)
        content = re.sub(r'mode:\s*\w+', 'mode: subagent', content)
        
        # เพิ่มป้ายกำกับ [Skill] ในชื่อ
        if 'name: ' in content and '[Skill]' not in content:
            content = content.replace('name: ', 'name: [Skill] ')
    
    with open(path, 'w', encoding='utf-8') as file:
        file.write(content)
    
    print(f"  - {f:30} -> {kind.upper()}")

print("\n✅ Categorization and Permission lockdown complete.")
