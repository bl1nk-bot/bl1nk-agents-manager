#!/usr/bin/env python3
import os
import sys
import json

def main():
    if len(sys.argv) < 2:
        target_dir = os.getcwd()
    else:
        target_dir = sys.argv[1]

    manifest_path = os.path.join(target_dir, "gemini-extension.json")
    
    print(f"🔍 Validating extension in: {target_dir}")
    errors = []

    # Check 1: Manifest Exists
    if not os.path.exists(manifest_path):
        errors.append("Missing 'gemini-extension.json'")
    else:
        # Check 2: Valid JSON
        try:
            with open(manifest_path, 'r') as f:
                data = json.load(f)
                
            # Check 3: Required Fields
            if "name" not in data:
                errors.append("Manifest missing 'name' field")
                
        except json.JSONDecodeError:
            errors.append("'gemini-extension.json' contains invalid JSON")

    # Report
    if errors:
        print("❌ Validation FAILED:")
        for e in errors:
            print(f" - {e}")
        sys.exit(1)
    else:
        print("✅ Validation PASSED! This looks like a valid extension.")

if __name__ == "__main__":
    main()
