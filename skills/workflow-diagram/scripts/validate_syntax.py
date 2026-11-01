#!/usr/bin/env python3
"""
Validate Mermaid diagram syntax in markdown files.
"""

import os
import re
import sys
import argparse
from pathlib import Path

def validate_mermaid_syntax(content):
    """Validate Mermaid syntax and return errors/warnings."""
    errors = []
    warnings = []
    
    # Extract Mermaid blocks
    pattern = r'```mermaid\n(.*?)\n```'
    blocks = re.findall(pattern, content, re.DOTALL)
    
    if not blocks:
        warnings.append("No Mermaid diagram blocks found")
        return errors, warnings, blocks
    
    for i, block in enumerate(blocks, 1):
        lines = block.strip().split('\n')
        
        # Check for basic structure
        if not any(line.strip().startswith(('stateDiagram', 'graph', 'sequenceDiagram', 'flowchart', 'pie', 'gantt')) for line in lines):
            errors.append(f"Block {i}: Missing diagram type declaration")
        
        # Check for common syntax errors
        for line_num, line in enumerate(lines, 1):
            line = line.strip()
            if not line or line.startswith('%') or line.startswith('%%'):
                continue
                
            # Check for unclosed quotes
            if line.count('"') % 2 != 0:
                errors.append(f"Block {i}, Line {line_num}: Unclosed quotes: {line}")
            
            # Check for invalid arrows in graphs
            if line.count('-->') > 0 and line.count('-[') > 0:
                errors.append(f"Block {i}, Line {line_num}: Mixed arrow syntax: {line}")
        
        # Check for proper state diagram syntax
        if 'stateDiagram' in block:
            state_transitions = [line for line in lines if '-->' in line and not line.strip().startswith('%%')]
            if len(state_transitions) < 2:
                warnings.append(f"Block {i}: State diagram has fewer than 2 transitions")
        
        # Check for proper graph syntax
        if 'graph ' in block or 'flowchart' in block:
            graph_connections = [line for line in lines if '-->' in line or '->' in line]
            if len(graph_connections) < 2:
                warnings.append(f"Block {i}: Graph has fewer than 2 connections")
    
    return errors, warnings, blocks

def validate_file(file_path):
    """Validate a single markdown file."""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
    except UnicodeDecodeError:
        print(f"❌ Error reading {file_path}: Encoding issues")
        return False
    except Exception as e:
        print(f"❌ Error reading {file_path}: {e}")
        return False
    
    errors, warnings, blocks = validate_mermaid_syntax(content)
    
    print(f"\n📄 Validating: {file_path}")
    print(f"📊 Found {len(blocks)} diagram(s)")
    
    if errors:
        print("\n❌ Errors:")
        for error in errors:
            print(f"   • {error}")
    
    if warnings:
        print("\n⚠️  Warnings:")
        for warning in warnings:
            print(f"   • {warning}")
    
    if not errors and not warnings:
        print("✅ All diagrams valid!")
    
    return len(errors) == 0

def main():
    parser = argparse.ArgumentParser(description='Validate Mermaid diagram syntax')
    parser.add_argument('input', help='Input markdown file or directory')
    parser.add_argument('-r', '--recursive', action='store_true', help='Search recursively')
    
    args = parser.parse_args()
    
    input_path = Path(args.input)
    valid_files = 0
    total_files = 0
    
    if input_path.is_file():
        total_files = 1
        if validate_file(input_path):
            valid_files = 1
    elif input_path.is_dir():
        pattern = '**/*.md' if args.recursive else '*.md'
        for md_file in input_path.glob(pattern):
            total_files += 1
            if validate_file(md_file):
                valid_files += 1
    else:
        print(f"❌ Error: {input_path} is not a valid file or directory")
        sys.exit(1)
    
    print(f"\n📈 Summary: {valid_files}/{total_files} files valid")
    
    if valid_files < total_files:
        sys.exit(1)

if __name__ == '__main__':
    main()