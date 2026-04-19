#!/usr/bin/env python3
import os
import toml
import yaml
import shutil
from pathlib import Path

def build_release():
    print("🚀 Starting Build Release process...")
    
    # Define paths
    root_dir = Path(__file__).parent.parent
    src_commands_dir = root_dir / "commands" / "blk"
    dist_dir = root_dir / "dist"
    dist_commands_dir = dist_dir / "commands"
    
    # Create/Clean dist directory
    if dist_dir.exists():
        shutil.rmtree(dist_dir)
    dist_commands_dir.mkdir(parents=True)
    
    print(f"📁 Source: {src_commands_dir}")
    print(f"📁 Destination: {dist_commands_dir}")
    
    # 1. Convert TOML commands to Markdown for universal usage
    for toml_file in src_commands_dir.glob("*.toml"):
        command_name = toml_file.stem
        print(f"  - Converting {command_name}.toml -> {command_name}.md")
        
        try:
            with open(toml_file, 'r', encoding='utf-8') as f:
                data = toml.load(f)
            
            description = data.get("description", "No description provided")
            prompt = data.get("prompt", "")
            
            # Construct frontmatter
            frontmatter = {
                "name": f"blk:{command_name}",
                "description": description,
                "version": "1.0.0"
            }
            
            # Handle arguments hint if present in prompt logic
            if "{{args}}" in prompt:
                frontmatter["argument-hint"] = "[arguments]"
            
            # Generate Markdown content
            md_content = "---\n"
            md_content += yaml.dump(frontmatter, sort_keys=False)
            md_content += "---\n\n"
            md_content += f"# {command_name.capitalize()} Command\n\n"
            md_content += prompt.strip()
            
            # Write to dist
            md_file = dist_commands_dir / f"{command_name}.md"
            with open(md_file, 'w', encoding='utf-8') as f:
                f.write(md_content)
                
        except Exception as e:
            print(f"  ❌ Error converting {toml_file.name}: {e}")

    # 2. Copy agents registry and system prompts to dist
    agents_src = root_dir / "agents"
    agents_dist = dist_dir / "agents"
    if agents_src.exists():
        print(f"📦 Copying agents registry to dist...")
        shutil.copytree(agents_src, agents_dist)

    print("\n✅ Build Release Complete!")
    print(f"📍 Distribution files ready at: {dist_dir}")

if __name__ == "__main__":
    build_release()
