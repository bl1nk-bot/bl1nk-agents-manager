#!/usr/bin/env python3

"""
Phase 1: Detect Duplicates (Python)
Scans Python project for duplicate modules, functions, structural similarities
"""

import json
import sys
import os
import hashlib
import ast
from pathlib import Path
from collections import defaultdict

def get_python_files(project_root):
    """Get all .py files excluding __pycache__ and venv"""
    files = []
    for root, dirs, filenames in os.walk(project_root):
        dirs[:] = [d for d in dirs if d not in ('__pycache__', 'venv', '.venv', '.git', 'node_modules')]
        for f in filenames:
            if f.endswith('.py'):
                files.append(os.path.join(root, f))
    return files

def get_file_hash(filepath):
    """MD5 hash of file content"""
    with open(filepath, 'rb') as f:
        return hashlib.md5(f.read()).hexdigest()

def parse_python_ast(filepath):
    """Parse Python file and extract structure"""
    try:
        with open(filepath, 'r', encoding='utf-8') as f:
            return ast.parse(f.read())
    except SyntaxError:
        return None

def extract_functions_and_classes(tree):
    """Extract function and class names"""
    if not tree:
        return []
    
    items = []
    for node in ast.walk(tree):
        if isinstance(node, (ast.FunctionDef, ast.ClassDef)):
            items.append(node.name)
    return items

def detect_python_duplicates(project_root, min_tokens=50):
    """Main detection logic"""
    files = get_python_files(project_root)
    
    results = {
        'timestamp': __import__('datetime').datetime.now().isoformat(),
        'python_files': files,
        'identical': [],
        'structural': [],
        'function_duplicates': []
    }
    
    # Identical file detection
    file_hashes = defaultdict(list)
    for filepath in files:
        try:
            file_hash = get_file_hash(filepath)
            file_hashes[file_hash].append(filepath)
        except Exception as e:
            print(f"Error hashing {filepath}: {e}", file=sys.stderr)
    
    for file_hash, file_list in file_hashes.items():
        if len(file_list) > 1:
            results['identical'].append({
                'hash': file_hash,
                'files': file_list,
                'size': os.path.getsize(file_list[0])
            })
    
    # Structural similarity
    structures = defaultdict(list)
    for filepath in files:
        tree = parse_python_ast(filepath)
        if tree:
            items = extract_functions_and_classes(tree)
            struct_key = tuple(sorted(items))
            structures[struct_key].append(filepath)
    
    for struct_key, file_list in structures.items():
        if len(file_list) > 1 and len(struct_key) >= min_tokens // 10:
            results['structural'].append({
                'functions_classes': list(struct_key),
                'files': file_list
            })
    
    return results

if __name__ == '__main__':
    project_root = sys.argv[1] if len(sys.argv) > 1 else os.getcwd()
    min_tokens = int(sys.argv[2]) if len(sys.argv) > 2 else 50
    
    duplicates = detect_python_duplicates(project_root, min_tokens)
    print(json.dumps(duplicates, indent=2))
