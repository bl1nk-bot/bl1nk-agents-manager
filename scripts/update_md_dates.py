#!/usr/bin/env python3
"""
Script to update the date in markdown files to reflect the actual git commit date.
This fixes the issue where markdown files have hardcoded dates that don't reflect
the actual last modification date from git history.
"""

import os
import subprocess
import re
from datetime import datetime
from pathlib import Path


def get_git_commit_date(file_path):
    """Get the last commit date for a file using git."""
    try:
        result = subprocess.run([
            'git', 'log', '-1', '--date=format:%b %d, %Y', 
            '--pretty=format:%ad', '--', str(file_path)
        ], cwd=os.getcwd(), capture_output=True, text=True, check=True)
        return result.stdout.strip()
    except subprocess.CalledProcessError:
        # If git command fails, return current date
        return datetime.now().strftime('%b %d, %Y')


def update_markdown_header(file_path):
    """Update the project status header in a markdown file with the git commit date."""
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()

    # Pattern to match the project status header with date
    pattern = r'(## 📌 Project Status \(.*?\))'
    
    # Find the current header
    match = re.search(pattern, content)
    if not match:
        print(f"No project status header found in {file_path}")
        return False

    current_header = match.group(1)
    
    # Get the actual git commit date for this file
    git_date = get_git_commit_date(file_path)
    new_header = f"## 📌 Project Status ({git_date})"
    
    if current_header == new_header:
        print(f"Date already correct in {file_path}")
        return False
    
    # Replace the header
    updated_content = content.replace(current_header, new_header, 1)
    
    # Write back to file
    with open(file_path, 'w', encoding='utf-8') as f:
        f.write(updated_content)
    
    print(f"Updated date in {file_path}: {current_header} -> {new_header}")
    return True


def main():
    """Main function to process all markdown files in the project."""
    project_root = Path(__file__).parent.parent
    
    # Find all markdown files in the project
    md_files = list(project_root.rglob("*.md"))
    
    # Filter files that have the project status header
    files_to_update = []
    for md_file in md_files:
        try:
            with open(md_file, 'r', encoding='utf-8') as f:
                content = f.read(200)  # Read first 200 chars to check for header
                if "## 📌 Project Status" in content:
                    files_to_update.append(md_file)
        except UnicodeDecodeError:
            continue  # Skip binary files
    
    print(f"Found {len(files_to_update)} markdown files with project status header")
    
    updated_count = 0
    for md_file in files_to_update:
        if update_markdown_header(md_file):
            updated_count += 1
    
    print(f"\nUpdated {updated_count} files with correct git commit dates")


if __name__ == "__main__":
    main()