#!/usr/bin/env python3
import os
import sys
import yaml
import json
import glob

REQUIRED_FIELDS = ['name', 'description', 'category']

def validate_frontmatter(file_path):
    """Check if .md file has valid frontmatter."""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
            
        if not content.startswith('---'):
            return False, "Missing frontmatter start '---'"
            
        parts = content.split('---', 2)
        if len(parts) < 3:
            return False, "Invalid frontmatter format"
            
        frontmatter = yaml.safe_load(parts[1])
        
        missing = [field for field in REQUIRED_FIELDS if field not in frontmatter]
        if missing:
            return False, f"Missing required fields: {', '.join(missing)}"
            
        return True, frontmatter
    except Exception as e:
        return False, str(e)

def validate_integrity(agents_dir, json_path):
    """Check consistency between files and agents.json."""
    issues = []
    
    # Load JSON
    try:
        with open(json_path, 'r') as f:
            registry = json.load(f)
            registered_files = {a['file'] for a in registry.get('agents', [])}
    except Exception as e:
        return [f"Failed to load agents.json: {e}"]

    # Check actual files
    md_files = glob.glob(os.path.join(agents_dir, "*.md"))
    
    for md_file in md_files:
        filename = os.path.basename(md_file)
        if filename == 'README.md':
            continue
            
        # 1. Frontmatter Validation
        valid, result = validate_frontmatter(md_file)
        if not valid:
            issues.append(f"[FILE] {filename}: {result}")
        
        # 2. Registry Validation
        if filename not in registered_files:
            issues.append(f"[REGISTRY] {filename}: Not registered in agents.json")

    return issues

def main():
    script_dir = os.path.dirname(os.path.abspath(__file__))
    extension_root = os.path.dirname(script_dir)
    agents_dir = os.path.join(extension_root, 'agents')
    json_path = os.path.join(agents_dir, 'agents.json')
    
    print(f"Validating agents in: {agents_dir}...")
    issues = validate_integrity(agents_dir, json_path)
    
    if issues:
        print("\n❌ Validation Failed found issues:")
        for issue in issues:
            print(f"  - {issue}")
        sys.exit(1)
    else:
        print("\n✅ All agents validated successfully!")
        sys.exit(0)

if __name__ == "__main__":
    main()
