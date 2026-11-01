#!/usr/bin/env python3
"""
Generate HTML files with embedded Mermaid diagrams from markdown files.
"""

import os
import re
import argparse
from pathlib import Path

def extract_mermaid_blocks(content):
    """Extract Mermaid diagram blocks from markdown content."""
    pattern = r'```mermaid\n(.*?)\n```'
    return re.findall(pattern, content, re.DOTALL)

def generate_html_diagrams(markdown_file, output_dir):
    """Generate HTML file with embedded Mermaid diagrams."""
    with open(markdown_file, 'r', encoding='utf-8') as f:
        content = f.read()
    
    mermaid_blocks = extract_mermaid_blocks(content)
    
    html_template = f"""
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{Path(markdown_file).stem} Workflow Diagrams</title>
    <script src="https://cdn.jsdelivr.net/npm/mermaid/dist/mermaid.min.js"></script>
    <style>
        body {{
            font-family: Arial, sans-serif;
            max-width: 1200px;
            margin: 0 auto;
            padding: 20px;
        }}
        .diagram {{
            margin: 30px 0;
            border: 1px solid #ddd;
            border-radius: 8px;
            padding: 20px;
            background-color: #f9f9f9;
        }}
        .diagram pre {{
            margin: 0;
        }}
        h1, h2, h3 {{
            color: #333;
        }}
    </style>
</head>
<body>
    <h1>{Path(markdown_file).stem} Workflow Diagrams</h1>
"""
    
    for i, block in enumerate(mermaid_blocks, 1):
        html_template += f"""
    <div class="diagram">
        <h2>Diagram {i}</h2>
        <div class="mermaid">
{block}
        </div>
    </div>
"""
    
    html_template += """
    <script>
        mermaid.initialize({ startOnLoad: true });
    </script>
</body>
</html>
"""
    
    output_file = Path(output_dir) / f"{Path(markdown_file).stem}.html"
    with open(output_file, 'w', encoding='utf-8') as f:
        f.write(html_template)
    
    print(f"Generated HTML: {output_file}")
    return len(mermaid_blocks)

def main():
    parser = argparse.ArgumentParser(description='Generate HTML diagrams from markdown')
    parser.add_argument('input', help='Input markdown file or directory')
    parser.add_argument('-o', '--output', default='./output', help='Output directory')
    
    args = parser.parse_args()
    
    input_path = Path(args.input)
    output_dir = Path(args.output)
    output_dir.mkdir(exist_ok=True)
    
    if input_path.is_file():
        diagrams = generate_html_diagrams(input_path, output_dir)
        print(f"Found {diagrams} diagrams in {input_path}")
    elif input_path.is_dir():
        total_diagrams = 0
        for md_file in input_path.glob('**/*.md'):
            diagrams = generate_html_diagrams(md_file, output_dir)
            total_diagrams += diagrams
            print(f"Found {diagrams} diagrams in {md_file}")
        print(f"Total: {total_diagrams} diagrams processed")

if __name__ == '__main__':
    main()