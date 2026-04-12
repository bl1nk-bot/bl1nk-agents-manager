#!/usr/bin/env bash
# ==============================================================================
# review.sh - Code review wrapper script
#
# Usage:
#   ./scripts/review.sh                  # Review local uncommitted changes
#   ./scripts/review.sh 123              # Review PR #123
#   ./scripts/review.sh 123 --comment    # Review PR + post inline comments
#   ./scripts/review.sh src/foo.rs       # Review specific file
#   ./scripts/review.sh https://github.com/.../pull/123
# ==============================================================================

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info()  { echo -e "${BLUE}[INFO]${NC}  $*"; }
log_ok()    { echo -e "${GREEN}[OK]${NC}    $*"; }
log_warn()  { echo -e "${YELLOW}[WARN]${NC}  $*"; }
log_error() { echo -e "${RED}[ERROR]${NC} $*"; }

# Determine review target
TARGET=""
COMMENT_FLAG=""

for arg in "$@"; do
    if [[ "$arg" == "--comment" ]]; then
        COMMENT_FLAG="--comment"
    else
        TARGET="$arg"
    fi
done

echo "========================================"
echo "  BL1NK Agents - Code Review"
echo "========================================"
echo ""

if [[ -z "$TARGET" ]]; then
    log_info "Reviewing local uncommitted changes..."

    # Check for changes
    if [[ -z "$(git diff)" ]] && [[ -z "$(git diff --staged)" ]]; then
        log_warn "No uncommitted changes found."
        exit 0
    fi

    echo ""
    log_info "Staged changes:"
    git diff --staged --stat 2>/dev/null || true
    echo ""
    log_info "Unstaged changes:"
    git diff --stat 2>/dev/null || true
    echo ""

    log_info "Use Qwen Code to review: /review"

elif [[ "$TARGET" =~ ^[0-9]+$ ]] || [[ "$TARGET" =~ github.com ]]; then
    log_info "Reviewing PR: $TARGET $COMMENT_FLAG"
    echo ""
    log_info "Use Qwen Code to review: /review $TARGET $COMMENT_FLAG"

else
    log_info "Reviewing file: $TARGET"
    echo ""
    git diff HEAD -- "$TARGET" || true
    echo ""
    log_info "Use Qwen Code to review: /review $TARGET"
fi

echo ""
echo "========================================"
echo "  Review complete!"
echo "========================================"
