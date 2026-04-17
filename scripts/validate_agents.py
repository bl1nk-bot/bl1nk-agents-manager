import os
import yaml
import json
import glob
import sys

def validate_file(path, is_skill=False):
    """Validate a single .md file content."""
    with open(path, 'r', encoding='utf-8') as f:
        try:
            content = f.read()
        except UnicodeDecodeError:
            return [f"Encoding error in {path}"]
    
    if not content.startswith('---'):
        return None if is_skill else [f"Missing frontmatter in Agent file {path}"]
    
    try:
        parts = content.split('---')
        if len(parts) < 3:
            return [f"Invalid frontmatter structure in {path}"]
            
        data = yaml.safe_load(parts[1])
        if not data:
            return [f"Empty frontmatter in {path}"]
            
        errors = []
        # name และ description ต้องมีเสมอ
        required = ["name", "description"]
        for field in required:
            if field not in data:
                errors.append(f"Missing required field '{field}' in {path}")

        # ถ้าเป็น Agent (อยู่ใน agents/) -> บังคับ 4 ฟิลด์
        if "agents/" in path:
            for field in ["mode", "tool"]:
                if field not in data:
                    errors.append(f"Agent requires field '{field}' in {path}")
        
        return errors
    except Exception as e:
        return [f"YAML Parse error in {path}: {e}"]

def main():
    print("🔍 Validating BL1NK Workforce Structure & Specs...")
    all_errors = []
    
    # 1. ตรวจสอบ Agents (Flat files)
    if os.path.exists('agents'):
        for f in glob.glob("agents/*.md"):
            if os.path.basename(f).upper() in ['README.MD', 'TODO.MD']: continue
            errs = validate_file(f, is_skill=False)
            if errs: all_errors.extend(errs)

    # 2. ตรวจสอบ Skills (Directories + SKILL.md)
    if os.path.exists('skills'):
        for skill_dir in os.listdir('skills'):
            dir_path = os.path.join('skills', skill_dir)
            if not os.path.isdir(dir_path): continue
            
            # กฎเหล็ก: ต้องมี SKILL.md
            skill_file = os.path.join(dir_path, 'SKILL.md')
            if not os.path.exists(skill_file):
                all_errors.append(f"Missing mandatory 'SKILL.md' in skill directory: {dir_path}")
                continue
                
            errs = validate_file(skill_file, is_skill=True)
            if errs: all_errors.extend(errs)

    if all_errors:
        print("\n🚩 Structure or Specification Errors:")
        for err in all_errors: print(f"❌ {err}")
        sys.exit(1)
    else:
        print("\n✨ All agents and skills follow the mandatory structure and rules!")
        sys.exit(0)

if __name__ == "__main__":
    main()
