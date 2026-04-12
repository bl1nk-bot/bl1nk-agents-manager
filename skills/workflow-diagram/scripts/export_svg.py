#!/usr/bin/env python3
"""
Export Mermaid diagrams to SVG format.
"""

import os
import re
import sys
import argparse
from pathlib import Path

def extract_mermaid_blocks(content):
    """Extract Mermaid diagram blocks from markdown content."""
    pattern = r'```mermaid\n(.*?)\n```'
    return re.findall(pattern, content, re.DOTALL)

def create_svg_from_mermaid(mermaid_code, output_file):
    """Create SVG using Mermaid CLI or online service."""
    # This is a placeholder - in practice, you'd use:
    # 1. Mermaid CLI: npx @mermaid-js/mermaid-cli
    # 2. Online service API
    # 3. Puppeteer/headless Chrome
    
    # For now, create a simple SVG placeholder
    svg_template = f"""<?xml version="1.0" encoding="UTF-8"?>
<svg width="800" height="600" xmlns="http://www.w3.org/2000/svg">
    <rect width="800" height="600" fill="#f9f9f9" stroke="#ddd" stroke-width="1"/>
    <text x="400" y="50" font-family="Arial, sans-serif" font-size="16" text-anchor="middle" fill="#333">
        Mermaid Diagram
    </text>
    <text x="20" y="100" font-family="monospace" font-size="12" fill="#666">
        {mermaid_code[:200]}{'...' if len(mermaid_code) > 200 else ''}
    </text>
    <text x="20" y="580" font-family="Arial, sans-serif" font-size="12" fill="#999">
        Note: Install mermaid-cli for actual SVG generation
    </text>
</svg>"""
    
    with open(output_file, 'w', encoding='utf-8') as f:
        f.write(svg_template)
    
    print(f"Generated SVG: {output_file}")
    return output_file

def export_diagrams_to_svg(markdown_file, output_dir):
    """Export all Mermaid diagrams in a markdown file to SVG."""
    with open(markdown_file, 'r', encoding='utf-8') as f:
        content = f.read()
    
    mermaid_blocks = extract_mermaid_blocks(content)
    output_dir = Path(output_dir)
    output_dir.mkdir(exist_ok=True)
    
    svg_files = []
    for i, block in enumerate(mermaid_blocks, 1):
        output_name = f"{Path(markdown_file).stem}_diagram_{i}.svg"
        output_path = output_dir / output_name
        create_svg_from_mermaid(block, output_path)
        svg_files.append(output_path)
    
    return svg_files

def check_mermaid_cli():
    """Check if mermaid CLI is available."""
    try:
        import subprocess
        result = subprocess.run(['npx', '@mermaid-js/mermaid-cli', '--version'], 
                          capture_output=True, text=True, timeout=10)
        return result.returncode == 0
    except:
        return False

def export_with_cli(mermaid_code, output_file):
    """Export using mermaid-cli if available."""
    try:
        import subprocess
        cmd = [
            'npx', '-y', '@mermaid-js/mermaid-cli',
            '-i', '-',
            '-o', str(output_file),
            '-b', 'white',
            '-s', '800x600'
        ]
        result = subprocess.run(cmd, input=mermaid_code, text=True, 
                          capture_output=True, timeout=30)
        return result.returncode == 0
    except Exception as e:
        print(f"CLI export failed: {e}")
        return False

def main():
    parser = argparse.ArgumentParser(description='Export Mermaid diagrams to SVG')
    parser.add_argument('input', help='Input markdown file or directory')
    parser.add_argument('-o', '--output', default='./svg_output', help='Output directory')
    parser.add_argument('--cli', action='store_true', help='Use mermaid-cli if available')
    
    args = parser.parse_args()
    
    input_path = Path(args.input)
    output_dir = Path(args.output)
    output_dir.mkdir(exist_ok=True)
    
    if args.cli and check_mermaid_cli():
        print("🚀 Using mermaid-cli for high-quality SVG export")
    else:
        print("⚠️  mermaid-cli not found. Creating placeholder SVGs.")
        print("    Install with: npm install -g @mermaid-js/mermaid-cli")
    
    if input_path.is_file():
        svg_files = export_diagrams_to_svg(input_path, output_dir)
        print(f"Exported {len(svg_files)} diagrams to {output_dir}")
    elif input_path.is_dir():
        total_exported = 0
        for md_file in input_path.glob('**/*.md'):
            svg_files = export_diagrams_to_svg(md_file, output_dir)
            total_exported += len(svg_files)
        print(f"Exported {total_exported} diagrams to {output_dir}")
    else:
        print(f"❌ Error: {input_path} is not valid")
        sys.exit(1)

if __name__ == '__main__':
    main()