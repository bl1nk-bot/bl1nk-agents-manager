#!/usr/bin/env bash
#
# Auto-labels GitHub issues/PRs using Bl1nk label system.
# - Auto-detects from title/body with word boundaries
# - Applies defaults for required fields (stage, type, p)
# - Calculates size from line changes (PR only)
# - Supports presets and manual overrides
# - **Agent labels are NOT auto‑detected** – user adds manually
#
# Usage: ./edit-issue-labels.sh [--preset PRESET] [--add-label LABEL] [--remove-label LABEL]
#

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LABELS_JSON="${SCRIPT_DIR}/../.github/labels.json"

if [[ ! -f "$LABELS_JSON" ]]; then
  echo "Error: $LABELS_JSON not found" >&2
  exit 1
fi

# Read from event payload
ISSUE=$(jq -r '.issue.number // .pull_request.number // .inputs.issue_number // empty' "${GITHUB_EVENT_PATH:?GITHUB_EVENT_PATH not set}")
if ! [[ "$ISSUE" =~ ^[0-9]+$ ]]; then
  echo "Error: no issue/PR number in event payload" >&2
  exit 1
fi

# Detect if it's a PR or issue
IS_PR=$(jq -r 'if .pull_request then "true" else "false" end' "${GITHUB_EVENT_PATH}")

# Get title and body
TITLE=$(jq -r '.issue.title // .pull_request.title // ""' "${GITHUB_EVENT_PATH}")
BODY=$(jq -r '.issue.body // .pull_request.body // ""' "${GITHUB_EVENT_PATH}")
FULL_TEXT="$TITLE $BODY"

# Get additions/deletions for size calculation (PR only)
ADDITIONS=$(jq -r '.pull_request.additions // 0' "${GITHUB_EVENT_PATH}")
DELETIONS=$(jq -r '.pull_request.deletions // 0' "${GITHUB_EVENT_PATH}")
TOTAL_CHANGES=$((ADDITIONS + DELETIONS))

ADD_LABELS=()
REMOVE_LABELS=()
DETECTED_LABELS=()

# Load presets
declare -A PRESETS
while IFS='=' read -r key value; do
  PRESETS[$key]=$value
done < <(jq -r '.presets | to_entries[] | "\(.key)=\(.value | join(" "))"' "$LABELS_JSON")

# Function to check if label exists
label_exists() {
  local label=$1
  gh label list --limit 500 --json name --jq '.[].name' | grep -qxF "$label"
}

# Function to add label safely
add_label_safe() {
  local label=$1
  if label_exists "$label"; then
    ADD_LABELS+=("$label")
    DETECTED_LABELS+=("$label")
  else
    echo "⚠️  Label '$label' does not exist, skipping" >&2
  fi
}

# Parse command line arguments
MANUAL_PRESET=""
MANUAL_ADD=()
MANUAL_REMOVE=()

while [[ $# -gt 0 ]]; do
  case $1 in
    --preset)
      MANUAL_PRESET="$2"
      shift 2
      ;;
    --add-label)
      MANUAL_ADD+=("$2")
      shift 2
      ;;
    --remove-label)
      MANUAL_REMOVE+=("$2")
      shift 2
      ;;
    *)
      echo "Error: unknown argument '$1'" >&2
      exit 1
      ;;
  esac
done

# Apply manual preset if provided
if [[ -n "$MANUAL_PRESET" ]]; then
  if [[ -z "${PRESETS[$MANUAL_PRESET]:-}" ]]; then
    echo "Error: unknown preset '$MANUAL_PRESET'" >&2
    exit 1
  fi
  for label in ${PRESETS[$MANUAL_PRESET]}; do
    add_label_safe "$label"
  done
else
  # Auto-detect labels from title/body
  echo "🔍 Auto-detecting labels from title and body..."

  # ---------- TYPE ----------
  TYPE_PATTERNS=$(jq -r '.autoDetect.patterns.type | to_entries[] | "\(.key):\(.value | join("|"))"' "$LABELS_JSON")
  TYPE_FOUND=false
  while IFS=':' read -r type_label patterns; do
    if echo "$FULL_TEXT" | grep -iqE "$patterns"; then
      add_label_safe "type:$type_label"
      TYPE_FOUND=true
      break
    fi
  done <<< "$TYPE_PATTERNS"

  # ---------- STAGE ----------
  STAGE_FOUND=false
  if echo "$FULL_TEXT" | grep -iqE '\b(plan|planning|spec|specification)\b'; then
    add_label_safe "stage:plan"
    STAGE_FOUND=true
  elif echo "$FULL_TEXT" | grep -iqE '\b(act|implement|doing|development)\b'; then
    add_label_safe "stage:act"
    STAGE_FOUND=true
  elif echo "$FULL_TEXT" | grep -iqE '\b(test|testing|qa)\b'; then
    add_label_safe "stage:test"
    STAGE_FOUND=true
  elif echo "$FULL_TEXT" | grep -iqE '\b(doc|document|documentation)\b'; then
    add_label_safe "stage:doc"
    STAGE_FOUND=true
  elif echo "$FULL_TEXT" | grep -iqE '\b(review|reviewing)\b'; then
    add_label_safe "stage:review"
    STAGE_FOUND=true
  elif echo "$FULL_TEXT" | grep -iqE '\b(finalize|finalized|done)\b'; then
    add_label_safe "stage:finalize"
    STAGE_FOUND=true
  fi

  if [[ "$STAGE_FOUND" == false ]]; then
    DEFAULT_STAGE=$(jq -r '.autoDetect.defaults.stage' "$LABELS_JSON")
    add_label_safe "$DEFAULT_STAGE"
  fi

  # ---------- SIZE ----------
  SIZE_FOUND=false
  if [[ "$IS_PR" == "true" ]]; then
    # Calculate size from line changes
    if [[ $TOTAL_CHANGES -le 50 ]]; then
      add_label_safe "size:xs"
      SIZE_FOUND=true
    elif [[ $TOTAL_CHANGES -le 150 ]]; then
      add_label_safe "size:s"
      SIZE_FOUND=true
    elif [[ $TOTAL_CHANGES -le 300 ]]; then
      add_label_safe "size:m"
      SIZE_FOUND=true
    elif [[ $TOTAL_CHANGES -le 600 ]]; then
      add_label_safe "size:l"
      SIZE_FOUND=true
    elif [[ $TOTAL_CHANGES -le 1200 ]]; then
      add_label_safe "size:xl"
      SIZE_FOUND=true
    elif [[ $TOTAL_CHANGES -le 3000 ]]; then
      add_label_safe "size:xxl"
      SIZE_FOUND=true
    else
      add_label_safe "size:xxxl"
      SIZE_FOUND=true
    fi
    echo "📊 PR size: $TOTAL_CHANGES lines changed"
  else
    # For issues, detect from keywords
    SIZE_PATTERNS=$(jq -r '.autoDetect.patterns.size | to_entries[] | "\(.key):\(.value | join("|"))"' "$LABELS_JSON")
    while IFS=':' read -r size_label patterns; do
      if echo "$FULL_TEXT" | grep -iqE "$patterns"; then
        add_label_safe "size:$size_label"
        SIZE_FOUND=true
        break
      fi
    done <<< "$SIZE_PATTERNS"
  fi

  if [[ "$SIZE_FOUND" == false ]]; then
    DEFAULT_SIZE=$(jq -r '.autoDetect.defaults.size' "$LABELS_JSON")
    add_label_safe "$DEFAULT_SIZE"
  fi

  # ---------- PRIORITY ----------
  P_FOUND=false
  P_PATTERNS=$(jq -r '.autoDetect.patterns.p | to_entries[] | "\(.key):\(.value | join("|"))"' "$LABELS_JSON")
  while IFS=':' read -r p_label patterns; do
    if echo "$FULL_TEXT" | grep -iqE "$patterns"; then
      add_label_safe "p:$p_label"
      P_FOUND=true
      break
    fi
  done <<< "$P_PATTERNS"

  if [[ "$P_FOUND" == false ]]; then
    DEFAULT_P=$(jq -r '.autoDetect.defaults.p' "$LABELS_JSON")
    add_label_safe "$DEFAULT_P"
  fi

  # ---------- LANGUAGE ----------
  LANG_PATTERNS=$(jq -r '.autoDetect.patterns.lang | to_entries[] | "\(.key):\(.value | join("|"))"' "$LABELS_JSON")
  while IFS=':' read -r lang_label patterns; do
    if echo "$FULL_TEXT" | grep -iqE "$patterns"; then
      add_label_safe "lang:$lang_label"
      break
    fi
  done <<< "$LANG_PATTERNS"

  # ---------- ENVIRONMENT ----------
  ENV_PATTERNS=$(jq -r '.autoDetect.patterns.env | to_entries[] | "\(.key):\(.value | join("|"))"' "$LABELS_JSON")
  while IFS=':' read -r env_label patterns; do
    if echo "$FULL_TEXT" | grep -iqE "$patterns"; then
      add_label_safe "env:$env_label"
      break
    fi
  done <<< "$ENV_PATTERNS"

  # ---------- CONSTRAINT ----------
  if echo "$FULL_TEXT" | grep -iqE '\b(mobile|mobile-first|responsive|ios|android)\b'; then
    add_label_safe "constraint:mobile"
  fi

  # ---------- PLATFORM (เพิ่มเติม) ----------
  PLAT_PATTERNS=$(jq -r '.autoDetect.patterns.plat | to_entries[] | "\(.key):\(.value | join("|"))"' "$LABELS_JSON")
  while IFS=':' read -r plat_label patterns; do
    if echo "$FULL_TEXT" | grep -iqE "$patterns"; then
      add_label_safe "plat:$plat_label"
      break
    fi
  done <<< "$PLAT_PATTERNS"

  # ---------- REVISION (เพิ่มเติม) ----------
  REV_PATTERNS=$(jq -r '.autoDetect.patterns.rev | to_entries[] | "\(.key):\(.value | join("|"))"' "$LABELS_JSON")
  while IFS=':' read -r rev_label patterns; do
    if echo "$FULL_TEXT" | grep -iqE "$patterns"; then
      add_label_safe "rev:$rev_label"
      break
    fi
  done <<< "$REV_PATTERNS"

  # ---------- BUG (other) ----------
  if echo "$FULL_TEXT" | grep -iqE '\b(bug|issue|error|broken|crash)\b'; then
    add_label_safe "Bug"
  fi
fi

# Add manual labels
for label in "${MANUAL_ADD[@]}"; do
  add_label_safe "$label"
done

# Add manual remove labels
for label in "${MANUAL_REMOVE[@]}"; do
  REMOVE_LABELS+=("$label")
done

# Remove duplicates
ADD_LABELS=($(printf '%s\n' "${ADD_LABELS[@]}" | sort -u))
REMOVE_LABELS=($(printf '%s\n' "${REMOVE_LABELS[@]}" | sort -u))

if [[ ${#ADD_LABELS[@]} -eq 0 && ${#REMOVE_LABELS[@]} -eq 0 ]]; then
  echo "ℹ️  No labels to apply"
  exit 0
fi

# Build gh command - works for both issues and PRs
GH_ARGS=("issue" "edit" "$ISSUE")

for label in "${ADD_LABELS[@]}"; do
  GH_ARGS+=("--add-label" "$label")
done

for label in "${REMOVE_LABELS[@]}"; do
  GH_ARGS+=("--remove-label" "$label")
done

gh "${GH_ARGS[@]}"

if [[ ${#ADD_LABELS[@]} -gt 0 ]]; then
  echo "✅ Added: ${ADD_LABELS[*]}"
fi
if [[ ${#REMOVE_LABELS[@]} -gt 0 ]]; then
  echo "❌ Removed: ${REMOVE_LABELS[*]}"
fi
