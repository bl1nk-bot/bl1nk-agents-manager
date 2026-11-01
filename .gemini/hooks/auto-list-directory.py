#!/usr/bin/env python3
import os
import json
import sys

# Configuration: Folders/Files to ignore
IGNORE_PATTERNS = {
    '.git', 'node_modules', '__pycache__', '.DS_Store', 
    'dist', 'build', 'coverage', '.venv', 'venv'
}

def get_directory_structure(root_dir):
    structure = []
    try:
        # List directories and files
        with os.scandir(root_dir) as entries:
            # Sort: Directories first, then files
            sorted_entries = sorted(entries, key=lambda e: (not e.is_dir(), e.name.lower()))
            
            for entry in sorted_entries:
                if entry.name in IGNORE_PATTERNS:
                    continue
                    
                if entry.is_dir():
                    structure.append(f"[DIR]  {entry.name}/")
                else:
                    structure.append(f"       {entry.name}")
                    
    except PermissionError:
        return ["(Permission Denied)"]
    except Exception as e:
        return [f"(Error: {str(e)})"]

    return structure

def main():
    # Get current working directory from environment or OS
    cwd = os.environ.get('GEMINI_PROJECT_DIR', os.getcwd())
    
    files = get_directory_structure(cwd)
    file_list_str = "\n".join(files)
    
    # Limit output length to prevent context overflow (approx 50 lines)
    if len(files) > 50:
        file_list_str = "\n".join(files[:50]) + f"\n... ({len(files) - 50} more items)"

    # Construct the context message
    context_message = f"""
I have listed the files in the current directory for you:
Current Directory: {cwd}

{file_list_str}
"""

    # Output JSON for Gemini CLI Hook System
    output = {
        "hookSpecificOutput": {
            "hookEventName": "SessionStart",
            "additionalContext": context_message
        },
        # Optional: Add a visible system message in the chat
        "systemMessage": f"📂 Auto-listed {len(files)} items in current directory."
    }
    
    print(json.dumps(output))

if __name__ == "__main__":
    main()
