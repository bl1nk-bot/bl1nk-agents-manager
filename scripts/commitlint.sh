#!/usr/bin/env bash
# ==============================================================================
# commitlint.sh - ตรวจสอบ commit messages ตาม conventional commit convention
#
# Convention:
#   type(scope): description
#
# Types ที่อนุญาต:
#   feat, fix, docs, style, refactor, perf, test, chore,
#   security, conductor, ci, build, revert
#
# Rules:
#   1. บรรทัดแรกไม่เกิน 72 ตัวอักษร
#   2. ต้องมี type ที่ถูกต้อง
#   3. description ต้องไม่เป็น empty
#   4. type ต้องเป็นตัวพิมพ์เล็กเท่านั้น
#   5. scope (ถ้ามี) ต้องเป็นตัวพิมพ์เล็ก + ขีดกลาง
#
# Usage:
#   echo "feat(auth): add login endpoint" | ./scripts/commitlint.sh
#   ./scripts/commitlint.sh --file COMMIT_MSG
#   ./scripts/commitlint.sh --last           # ตรวจสอบ commit ล่าสุด
#   ./scripts/commitlint.sh --range v1..v2   # ตรวจสอบ range
# ==============================================================================

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

VALID_TYPES="feat|fix|docs|style|refactor|perf|test|chore|security|conductor|ci|build|revert"
MAX_SUBJECT_LENGTH=72
WARNINGS=0
ERRORS=0

usage() {
    cat <<EOF
Usage: $(basename "$0") [OPTIONS]

Options:
  --file <file>      อ่าน commit message จากไฟล์
  --last             ตรวจสอบ commit ล่าสุด
  --range <range>    ตรวจสอบ commits ในช่วง (e.g., v0.1.0..HEAD)
  --strict           fail ทันทีที่พบ error แรก
  -h, --help         แสดงวิธีใช้

Examples:
  echo "feat(auth): add login" | $(basename "$0")
  $(basename "$0") --file .git/COMMIT_EDITMSG
  $(basename "$0") --last
  $(basename "$0") --range HEAD~5..HEAD
EOF
    exit 0
}

log_pass() { echo -e "${GREEN}  ✓${NC} $*"; }
log_warn() { echo -e "${YELLOW}  ⚠${NC} $*"; ((WARNINGS++)) || true; }
log_fail() { echo -e "${RED}  ✗${NC} $*"; ((ERRORS++)) || true; }

# ==============================================================================
# ฟังก์ชันตรวจสอบ commit message เดียว
# ==============================================================================
lint_commit() {
    local message="$1"
    local short_hash="${2:-}"
    local has_error=0

    # อ่านบรรทัดแรก (subject line)
    local subject
    subject=$(echo "$message" | head -1)

    echo ""
    if [[ -n "$short_hash" ]]; then
        echo -e "${BLUE}--- [$short_hash] ---${NC}"
    fi
    echo "Subject: $subject"

    # Rule 1: subject line ไม่เกิน 72 ตัวอักษร
    if [[ ${#subject} -gt $MAX_SUBJECT_LENGTH ]]; then
        log_fail "Subject line ยาวเกิน $MAX_SUBJECT_LENGTH ตัวอักษร (${#subject} ตัว)"
        has_error=1
    else
        log_pass "Subject line length OK (${#subject} ตัวอักษร)"
    fi

    # Rule 2: subject line ต้องไม่เป็น empty
    if [[ -z "$subject" ]]; then
        log_fail "Subject line เป็น empty"
        has_error=1
        return $has_error
    fi

    # Rule 3: ต้องตรงกับ conventional commit format
    # Pattern: type(scope): description หรือ type: description
    if [[ "$subject" =~ ^([a-zA-Z]+)(\(([a-zA-Z0-9_-]+)\))?:\ (.+)$ ]]; then
        local type="${BASH_REMATCH[1]}"
        local scope="${BASH_REMATCH[3]}"
        local description="${BASH_REMATCH[4]}"

        # Rule 4: type ต้องเป็นตัวพิมพ์เล็ก
        if [[ "$type" != "${type,,}" ]]; then
            log_fail "Type ต้องเป็นตัวพิมพ์เล็กเท่านั้น: '$type' ควรเป็น '${type,,}'"
            has_error=1
        fi

        # Rule 5: type ต้องอยู่ในรายการที่อนุญาต
        if ! echo "$type" | grep -qE "^($VALID_TYPES)$"; then
            log_fail "Type '$type' ไม่อยู่ในรายการที่อนุญาต"
            log_info "Types ที่อนุญาต: $(echo "$VALID_TYPES" | tr '|' ', ')"
            has_error=1
        else
            log_pass "Type '$type' ถูกต้อง"
        fi

        # Rule 6: scope (ถ้ามี) ต้องเป็นตัวพิมพ์เล็ก
        if [[ -n "$scope" ]]; then
            if [[ "$scope" != "${scope,,}" ]]; then
                log_warn "Scope ควรเป็นตัวพิมพ์เล็ก: '$scope' -> '${scope,,}'"
            fi
        fi

        # Rule 7: description ต้องไม่เป็น empty
        if [[ -z "$description" ]]; then
            log_fail "Description เป็น empty"
            has_error=1
        else
            log_pass "Description มีเนื้อหา"
        fi

        # Rule 8: description ต้องไม่ลงท้ายด้วย .
        if [[ "$description" == *. ]]; then
            log_warn "Description ไม่ควรลงท้ายด้วย '.'"
        fi

    elif [[ "$subject" =~ ^(Merge|Initial) ]]; then
        # อนุญาต merge commits และ initial commit
        log_warn "Merge/Initial commit (ข้าม validation)"
    else
        log_fail "ไม่ตรงกับ conventional commit format: type(scope): description"
        log_info "ตัวอย่างที่ถูกต้อง:"
        echo -e "  ${GREEN}feat(auth): เพิ่ม login endpoint${NC}"
        echo -e "  ${GREEN}fix: แก้ไข buffer overflow${NC}"
        echo -e "  ${GREEN}docs(readme): อัปเดต quickstart${NC}"
        has_error=1
    fi

    return $has_error
}

# ==============================================================================
# Main Logic
# ==============================================================================
COMMIT_MESSAGE=""
FILE=""
LAST=false
RANGE=""
STRICT=false

while [[ $# -gt 0 ]]; do
    case "$1" in
        --file)    FILE="$2"; shift 2 ;;
        --last)    LAST=true; shift ;;
        --range)   RANGE="$2"; shift 2 ;;
        --strict)  STRICT=true; shift ;;
        -h|--help) usage ;;
        *)         log_fail "Unknown option: $1"; usage ;;
    esac
done

# อ่าน commit message
if [[ -n "$FILE" ]]; then
    COMMIT_MESSAGE=$(cat "$FILE")
elif [[ "$LAST" == true ]]; then
    COMMIT_MESSAGE=$(git log -1 --format="%B")
elif [[ -n "$RANGE" ]]; then
    # ตรวจสอบหลาย commits
    echo -e "${BLUE}========================================${NC}"
    echo -e "${BLUE}  Commit Lint: $RANGE${NC}"
    echo -e "${BLUE}========================================${NC}"

    local_errors=0
    while IFS='|' read -r hash subject; do
        [[ -z "$hash" ]] && continue

        local msg
        msg=$(git log -1 --format="%B" "$hash")

        if ! lint_commit "$msg" "$hash"; then
            ((local_errors++))
            if [[ "$STRICT" == true ]]; then
                echo ""
                echo -e "${RED}Strict mode: หยุดที่ error แรก${NC}"
                exit 1
            fi
        fi
    done < <(git log "$RANGE" --format="%H|%s")

    echo ""
    echo "========================================"
    echo "Results: $local_errors commits with errors"
    if [[ $local_errors -eq 0 ]]; then
        echo -e "${GREEN}All commits pass validation!${NC}"
    else
        echo -e "${RED}Found $local_errors commits with errors${NC}"
    fi
    exit $local_errors
elif [[ ! -t 0 ]]; then
    # อ่านจาก stdin
    COMMIT_MESSAGE=$(cat)
else
    echo -e "${RED}Error: ต้องระบุ commit message ผ่าน stdin, --file, --last, หรือ --range${NC}"
    usage
fi

# ตรวจสอบ commit เดียว
if [[ -n "$COMMIT_MESSAGE" ]]; then
    echo -e "${BLUE}========================================${NC}"
    echo -e "${BLUE}  Commit Lint${NC}"
    echo -e "${BLUE}========================================${NC}"

    if lint_commit "$COMMIT_MESSAGE"; then
        echo ""
        echo -e "${GREEN}========================================${NC}"
        echo -e "${GREEN}  ✓ Commit message ผ่าน validation!${NC}"
        echo -e "${GREEN}========================================${NC}"
        exit 0
    else
        echo ""
        echo -e "${RED}========================================${NC}"
        echo -e "${RED}  ✗ Commit message ไม่ผ่าน validation${NC}"
        echo -e "${RED}  Errors: $ERRORS | Warnings: $WARNINGS${NC}"
        echo -e "${RED}========================================${NC}"
        exit 1
    fi
fi
