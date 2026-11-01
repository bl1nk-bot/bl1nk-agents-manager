#!/usr/bin/env python3
import os
import sys
import json
import argparse

# Setup Paths
SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
# Move up from scripts/ -> extension-architect/ -> skills/ -> extension-creator/
EXTENSION_ROOT = os.path.abspath(os.path.join(SCRIPT_DIR, "../../../"))
ASSETS_DIR = os.path.join(EXTENSION_ROOT, "assets")

def load_template(filename):
    with open(os.path.join(ASSETS_DIR, filename), 'r') as f:
        return f.read()

def main():
    parser = argparse.ArgumentParser(description="Initialize a new Gemini Extension")
    parser.add_argument("name", help="Name of the extension (kebab-case)")
    parser.add_argument("--skill", help="Name of the initial skill (optional)")
    args = parser.parse_args()

    target_dir = args.name
    if os.path.exists(target_dir):
        print(f"❌ Error: Directory '{target_dir}' already exists.")
        sys.exit(1)

    print(f"🚀 Scaffolding extension: {args.name}...")
    
    # 1. Create Directories
    os.makedirs(os.path.join(target_dir, "commands"), exist_ok=True)
    if args.skill:
        os.makedirs(os.path.join(target_dir, "skills", args.skill, "scripts"), exist_ok=True)
    
    # 2. Create gemini-extension.json
    manifest_content = load_template("manifest-template.json").replace("{{EXTENSION_NAME}}", args.name)
    manifest_data = json.loads(manifest_content)
    
    if args.skill:
        manifest_data["skills"].append({
            "path": f"skills/{args.skill}",
            "name": args.skill
        })
    
    with open(os.path.join(target_dir, "gemini-extension.json"), "w") as f:
        json.dump(manifest_data, f, indent=2)

    # 3. Create SKILL.md (if requested)
    if args.skill:
        skill_content = load_template("skill-template.md") \
            .replace("{{SKILL_NAME}}", args.skill) \
            .replace("{{SKILL_DESCRIPTION}}", f"Skill for {args.name}")
        
        with open(os.path.join(target_dir, "skills", args.skill, "SKILL.md"), "w") as f:
            f.write(skill_content)

    print(f"✅ Extension '{args.name}' created successfully!")
    print(f"👉 Next step: Run 'gemini extensions link {os.path.abspath(target_dir)}'")

if __name__ == "__main__":
    main()
