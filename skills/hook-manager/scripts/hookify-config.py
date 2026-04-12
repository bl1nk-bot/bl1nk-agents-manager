#!/usr/bin/env python3
"""
Hookify Configuration Manager (Gemini Edition)
Best Practice: Manages the "disabled" list in .gemini/settings.json using native JSON handling.
"""

import json
import os
import sys

SETTINGS_FILE = os.path.join(".gemini", "settings.json")

def load_settings():
    if not os.path.exists(SETTINGS_FILE):
        print(f"⚠️  Settings file not found at {SETTINGS_FILE}")
        return {"hooks": {"disabled": []}}
    
    try:
        with open(SETTINGS_FILE, 'r', encoding='utf-8') as f:
            return json.load(f)
    except json.JSONDecodeError:
        print(f"❌ Error: {SETTINGS_FILE} is not valid JSON.")
        sys.exit(1)

def save_settings(data):
    # Create directory if needed
    os.makedirs(os.path.dirname(SETTINGS_FILE), exist_ok=True)
    with open(SETTINGS_FILE, 'w', encoding='utf-8') as f:
        json.dump(data, f, indent=2)
    print("\n✅ Configuration updated successfully!")

def get_all_hooks(settings):
    hooks = set()
    hooks_config = settings.get("hooks", {})
    
    for key, value in hooks_config.items():
        if key == "disabled":
            continue
        if isinstance(value, list):
            for matcher_group in value:
                for hook in matcher_group.get("hooks", []):
                    if "name" in hook:
                        hooks.add(hook["name"])
    return sorted(list(hooks))

def main():
    print("🔍 Reading Gemini settings...")
    settings = load_settings()
    
    all_hooks = get_all_hooks(settings)
    disabled_hooks = set(settings.get("hooks", {}).get("disabled", []))
    
    if not all_hooks:
        print(f"❌ No hooks defined in {SETTINGS_FILE}.")
        print("Add some hooks to 'BeforeTool', 'AfterModel', etc. first.")
        sys.exit(0)

    print("\n📋 Available Hooks:")
    print("-" * 50)
    print(f"{ 'ID':<5} {'NAME':<25} {'STATE':<10}")
    print("-" * 50)
    
    hook_map = {}
    for idx, name in enumerate(all_hooks, 1):
        state = "❌ DISABLED" if name in disabled_hooks else "✅ ENABLED"
        print(f"{idx:<5} {name:<25} {state:<10}")
        hook_map[str(idx)] = name

    print("-" * 50)
    selection = input("\nEnter the IDs of hooks to TOGGLE (comma separated, e.g., '1,3'): ").strip()
    
    if not selection:
        print("No changes made.")
        sys.exit(0)
    
    # Initialize disabled list if missing
    if "hooks" not in settings:
        settings["hooks"] = {}
    if "disabled" not in settings["hooks"]:
        settings["hooks"]["disabled"] = []
        
    current_disabled = set(settings["hooks"]["disabled"])
    changes_made = False

    for item in selection.split(','):
        item = item.strip()
        if item in hook_map:
            target = hook_map[item]
            if target in current_disabled:
                print(f"🔄 Enabling: {target}")
                current_disabled.remove(target)
            else:
                print(f"🔄 Disabling: {target}")
                current_disabled.add(target)
            changes_made = True
        else:
            print(f"⚠️  Invalid ID: {item} (Skipping)")

    if changes_made:
        settings["hooks"]["disabled"] = sorted(list(current_disabled))
        save_settings(settings)
    else:
        print("No valid changes selected.")

if __name__ == "__main__":
    main()
