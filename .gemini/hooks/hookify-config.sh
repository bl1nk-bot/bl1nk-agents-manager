#!/usr/bin/env bash
# Hookify Configuration Manager (Gemini Edition)
# Best Practice: Manages the "disabled" list in .gemini/settings.json using jq

set -e

SETTINGS_FILE=".gemini/settings.json"

# --- 1. Validation ---
if ! command -v jq &> /dev/null; then
    echo "❌ Error: 'jq' is required but not installed."
    echo "Please install it (e.g., 'winget install jqlang.jq' or 'brew install jq')."
    exit 1
fi

if [ ! -f "$SETTINGS_FILE" ]; then
    echo "⚠️  Settings file not found at $SETTINGS_FILE"
    echo "Creating a default one..."
    mkdir -p .gemini
    echo '{ "hooks": { "disabled": [] } }' > "$SETTINGS_FILE"
fi

echo "🔍 Reading Gemini settings..."

# --- 2. Extract Data ---
# Get all defined hook names from all event categories (excluding 'disabled')
ALL_HOOKS=$(jq -r '
  .hooks 
  | to_entries[] 
  | select(.key != "disabled") 
  | .value[]? 
  | .hooks[]? 
  | .name
' "$SETTINGS_FILE" | sort | uniq)

# Get currently disabled hooks
DISABLED_HOOKS=$(jq -r '.hooks.disabled[]? // empty' "$SETTINGS_FILE" | sort | uniq)

if [ -z "$ALL_HOOKS" ]; then
    echo "❌ No hooks defined in $SETTINGS_FILE."
    echo "Add some hooks to 'BeforeTool', 'AfterModel', etc. first."
    exit 0
fi

# --- 3. Interactive UI ---
echo -e "\n📋 Available Hooks:"
echo "------------------------------------------------"
printf "% -5s %-25s %-10s\n" "ID" "NAME" "STATE"
echo "------------------------------------------------"

declare -a HOOK_ARRAY
i=1

# Process each hook to determine state
while IFS= read -r hook_name; do
    if [ -z "$hook_name" ]; then continue; fi
    
    # Check if disabled
    state="✅ ENABLED"
    if echo "$DISABLED_HOOKS" | grep -qFx "$hook_name"; then
        state="❌ DISABLED"
    fi
    
    printf "% -5s %-25s %-10s\n" "$i" "$hook_name" "$state"
    HOOK_ARRAY[$i]="$hook_name"
    ((i++))
done <<< "$ALL_HOOKS"

echo "------------------------------------------------"
echo -e "\nEnter the IDs of hooks to TOGGLE (comma separated, e.g., '1,3'):"
read -r selection

if [ -z "$selection" ]; then
    echo "No changes made."
    exit 0
fi

# --- 4. Process Changes ---
IFS=',' read -ra ADDR <<< "$selection"
for id in "${ADDR[@]}"; do
    # Trim whitespace
    id=$(echo "$id" | xargs)
    
    # Validate number
    if ! [[ "$id" =~ ^[0-9]+$ ]] || [ "$id" -ge "$i" ] || [ "$id" -lt 1 ]; then
        echo "⚠️  Invalid ID: $id (Skipping)"
        continue
    fi

    target_hook="${HOOK_ARRAY[$id]}"
    
    # Determine action: If in disabled list -> Enable (remove). Else -> Disable (add).
    if echo "$DISABLED_HOOKS" | grep -qFx "$target_hook"; then
        echo "🔄 Enabling: $target_hook"
        # Remove from disabled list
        tmp=$(mktemp)
        jq --arg h "$target_hook" '.hooks.disabled -= [$h]' "$SETTINGS_FILE" > "$tmp" && mv "$tmp" "$SETTINGS_FILE"
    else
        echo "🔄 Disabling: $target_hook"
        # Add to disabled list (ensure 'disabled' array exists first)
        tmp=$(mktemp)
        jq --arg h "$target_hook" \
            '\
            if .hooks.disabled then . else .hooks.disabled = [] end 
            | .hooks.disabled += [$h] 
            | .hooks.disabled |= unique
        ' "$SETTINGS_FILE" > "$tmp" && mv "$tmp" "$SETTINGS_FILE"
    fi
done

echo -e "\n✅ Configuration updated successfully!"
