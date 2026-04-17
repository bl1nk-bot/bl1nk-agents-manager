import os
import json
import yaml
import argparse
import glob

def load_all_assets():
    """Load both agents and skills from their respective directories."""
    assets = {}
    directories = [('agents', 'Agent'), ('skills', 'Skill')]
    
    for base_dir, asset_type in directories:
        if not os.path.exists(base_dir): continue
        
        # Scan for .md files
        md_files = glob.glob(os.path.join(base_dir, "**/*.md"), recursive=True)
        for md_path in md_files:
            filename = os.path.basename(md_path)
            if filename in ['README.md', 'CHANGELOG.md']: continue
            
            with open(md_path, 'r', encoding='utf-8') as f:
                content = f.read()
                if content.startswith('---'):
                    try:
                        # Extract frontmatter
                        parts = content.split('---')
                        data = yaml.safe_load(parts[1])
                        name = data.get('name', os.path.splitext(filename)[0])
                        assets[name] = {
                            'name': name,
                            'type': asset_type,
                            'path': md_path,
                            'description': data.get('description', 'No description')
                        }
                    except: pass
    return assets

def cmd_list():
    assets = load_all_assets()
    print(f"{'TYPE':<10} {'NAME':<30} {'PATH'}")
    print("-" * 80)
    for name in sorted(assets.keys()):
        item = assets[name]
        print(f"{item['type']:<10} {item['name']:<30} {item['path']}")

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="BL1NK Asset Manager")
    parser.add_argument("command", choices=["list", "check"])
    args = parser.parse_args()
    
    if args.command == "list":
        cmd_list()
    elif args.command == "check":
        # Will be handled by validate_agents.py
        pass
