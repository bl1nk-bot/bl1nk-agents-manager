#!/usr/bin/env python3
import os
import json
import glob
import yaml

def get_default_frontmatter(filename):
    name = filename.replace('.md', '')
    return {
        'name': name,
        'description': f"Agent for {name.replace('-', ' ')}",
        'category': 'utility'
    }

def fix_frontmatter(file_path):
    """Ensure file has valid frontmatter."""
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    frontmatter = {}
    body = content

    if content.startswith('---'):
        try:
            parts = content.split('---', 2)
            if len(parts) >= 3:
                frontmatter = yaml.safe_load(parts[1]) or {}
                body = parts[2].lstrip()
        except:
            pass
    
    # Fill defaults
    defaults = get_default_frontmatter(os.path.basename(file_path))
    
    if 'name' not in frontmatter:
        frontmatter['name'] = defaults['name']
    if 'description' not in frontmatter:
        frontmatter['description'] = defaults['description']
    if 'category' not in frontmatter:
        frontmatter['category'] = defaults['category']
        
    # Reconstruct file
    new_content = f"---\n{yaml.dump(frontmatter, sort_keys=False)}---\n\n{body}"
    
    with open(file_path, 'w', encoding='utf-8') as f:
        f.write(new_content)
        
    return frontmatter

def update_registry(agents_dir, json_path):
    """Update agents.json with all found md files."""
    
    # Load existing
    try:
        with open(json_path, 'r') as f:
            data = json.load(f)
    except:
        data = {'agents': []}
        
    existing_map = {a['file']: a for a in data.get('agents', [])}
    
    # Scan files
    md_files = glob.glob(os.path.join(agents_dir, "*.md"))
    
    for md_file in md_files:
        filename = os.path.basename(md_file)
        if filename == 'README.md':
            continue
            
        # Fix file content first
        fm = fix_frontmatter(md_file)
        
        # Update registry data
        agent_entry = {
            'id': fm['name'],
            'name': fm['name'].replace('-', ' ').title(),
            'file': filename,
            'category': fm['category'],
            'description': fm['description']
        }
        
        # Merge with existing (preserve manual edits if any)
        if filename in existing_map:
            existing_map[filename].update(agent_entry)
        else:
            existing_map[filename] = agent_entry
            
    # Rebuild list
    data['agents'] = list(existing_map.values())
    
    # Save
    with open(json_path, 'w') as f:
        json.dump(data, f, indent=2)
        
    return len(data['agents'])

def main():
    script_dir = os.path.dirname(os.path.abspath(__file__))
    extension_root = os.path.dirname(script_dir)
    agents_dir = os.path.join(extension_root, 'agents')
    json_path = os.path.join(agents_dir, 'agents.json')
    
    print("Auto-fixing agents...")
    count = update_registry(agents_dir, json_path)
    print(f"✅ Registered {count} agents successfully!")

if __name__ == "__main__":
    main()
