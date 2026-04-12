#!/usr/bin/env bash
# ==============================================================================
# update-security.sh - ตรวจสอบและอัปเดต security dependencies
#
# ฟังก์ชัน:
#   1. รัน cargo audit (ถ้ามี cargo-audit)
#   2. ตรวจสอบ dependencies ที่ล้าสมัย
#   3. อัปเดต dependencies ไปเป็นเวอร์ชันล่าสุดที่ปลอดภัย
#   4. สร้าง security report
#
# Usage:
#   ./scripts/update-security.sh              # ตรวจสอบ + อัปเดตทั้งหมด
#   ./scripts/update-security.sh --check-only # ตรวจสอบอย่างเดียว ไม่อัปเดต
#   ./scripts/update-security.sh --report     # สร้าง report อย่างเดียว
# ==============================================================================

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

CHECK_ONLY=false
REPORT_ONLY=false
REPORT_DIR="docs/security"
REPORT_FILE="$REPORT_DIR/security-report-$(date +'%Y%m%d-%H%M%S').md"

usage() {
    cat <<EOF
Usage: $(basename "$0") [OPTIONS]

Options:
  --check-only   ตรวจสอบอย่างเดียว ไม่อัปเดต dependencies
  --report       สร้าง security report อย่างเดียว
  -h, --help     แสดงวิธีใช้

Examples:
  $(basename "$0")                  # ตรวจสอบ + อัปเดตทั้งหมด
  $(basename "$0") --check-only     # ตรวจสอบอย่างเดียว
  $(basename "$0") --report         # สร้าง report
EOF
    exit 0
}

log_info()  { echo -e "${BLUE}[INFO]${NC}  $*"; }
log_ok()    { echo -e "${GREEN}[OK]${NC}    $*"; }
log_warn()  { echo -e "${YELLOW}[WARN]${NC}  $*"; }
log_error() { echo -e "${RED}[ERROR]${NC} $*"; }
log_step()  { echo -e "${CYAN}[STEP]${NC}  $*"; }

# Parse arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        --check-only) CHECK_ONLY=true; shift ;;
        --report)     REPORT_ONLY=true; shift ;;
        -h|--help)    usage ;;
        *)            log_error "Unknown option: $1"; usage ;;
    esac
done

# สร้าง directory สำหรับ report
mkdir -p "$REPORT_DIR"

# ==============================================================================
# ส่วนที่ 1: ตรวจสอบ cargo-audit
# ==============================================================================
check_audit() {
    log_step "ตรวจสอบ dependencies ด้วย cargo-audit..."

    if command -v cargo-audit &> /dev/null; then
        log_info "Running cargo audit..."
        if cargo audit 2>&1 | tee "$REPORT_DIR/audit-output.txt"; then
            log_ok "No security vulnerabilities found"
            return 0
        else
            log_warn "พบ security vulnerabilities! ดูรายละเอียดใน $REPORT_DIR/audit-output.txt"
            return 1
        fi
    else
        log_warn "cargo-audit ไม่ติดตั้ง"
        log_info "ติดตั้งด้วย: cargo install cargo-audit"
        return 1
    fi
}

# ==============================================================================
# ส่วนที่ 2: ตรวจสอบ outdated dependencies
# ==============================================================================
check_outdated() {
    log_step "ตรวจสอบ outdated dependencies..."

    local outdated_file="$REPORT_DIR/outdated-deps.txt"
    cargo outdated 2>/dev/null | tee "$outdated_file" || {
        log_warn "cargo-outdated ไม่ติดตั้ง หรือพบข้อผิดพลาด"
        log_info "ติดตั้งด้วย: cargo install cargo-outdated"
        # ใช้ cargo update --dry-run แทน
        log_info "ใช้ cargo update --dry-run แทน..."
        cargo update --dry-run 2>&1 | tee "$outdated_file" || true
    }

    if [[ -s "$outdated_file" ]]; then
        log_info "พบ outdated dependencies:"
        head -20 "$outdated_file"
    else
        log_ok "All dependencies are up to date"
    fi
}

# ==============================================================================
# ส่วนที่ 3: อัปเดต dependencies
# ==============================================================================
update_dependencies() {
    if [[ "$CHECK_ONLY" == true ]]; then
        log_info "--check-only: ข้ามการอัปเดต"
        return 0
    fi

    log_step "อัปเดต dependencies ไปเวอร์ชันล่าสุด..."

    # อัปเดต Cargo.lock ไปเป็นเวอร์ชันที่ตรงกับ Cargo.toml
    cargo update 2>&1 | tail -5
    log_ok "Dependencies updated (Cargo.lock regenerated)"

    # ตรวจสอบว่า build ผ่านหลังอัปเดต
    log_info "Verify build หลังอัปเดต..."
    if cargo check --all-features --quiet; then
        log_ok "Build verification passed"
    else
        log_error "Build ล้มเหลวหลังอัปเดต dependencies!"
        return 1
    fi
}

# ==============================================================================
# ส่วนที่ 4: ตรวจสอบ insecure code patterns
# ==============================================================================
check_insecure_patterns() {
    log_step "ตรวจสอบ insecure code patterns..."

    local issues=0

    # ตรวจสอบ hardcoded secrets
    if grep -rn "password\s*=\s*\"" --include="*.rs" src/ 2>/dev/null; then
        log_warn "พบ hardcoded password ใน source code"
        ((issues++))
    fi

    # ตรวจสอบ API keys
    if grep -rn "api_key\s*=\s*\"" --include="*.rs" src/ 2>/dev/null; then
        log_warn "พบ hardcoded API key ใน source code"
        ((issues++))
    fi

    # ตรวจสอบ unwrap() ใน production code (ควรใช้ ? หรือ expect() แทน)
    local unwrap_count
    unwrap_count=$(grep -rn "\.unwrap()" --include="*.rs" src/ 2>/dev/null | wc -l || echo "0")
    if [[ "$unwrap_count" -gt 0 ]]; then
        log_warn "พบ .unwrap() $unwrap_count ครั้งใน src/ (พิจารณาใช้ ? หรือ expect())"
    fi

    # ตรวจสอบ unsafe blocks
    local unsafe_count
    unsafe_count=$(grep -rn "unsafe " --include="*.rs" src/ 2>/dev/null | wc -l || echo "0")
    if [[ "$unsafe_count" -gt 0 ]]; then
        log_warn "พบ unsafe block $unsafe_count ครั้งใน src/"
    fi

    if [[ $issues -eq 0 ]]; then
        log_ok "No insecure patterns found"
    else
        log_warn "พบ $issues insecure pattern(s)"
    fi
}

# ==============================================================================
# ส่วนที่ 5: สร้าง Security Report
# ==============================================================================
generate_report() {
    log_step "สร้าง security report: $REPORT_FILE"

    local current_version
    current_version=$(grep -E '^version\s*=' Cargo.toml | head -1 | sed -E 's/^version\s*=\s*"([^"]+)"/\1/')

    local commit_hash
    commit_hash=$(git rev-parse --short HEAD)

    cat > "$REPORT_FILE" <<EOF
# Security Report

**Date:** $(date -u +'%Y-%m-%d %H:%M:%S UTC')
**Version:** $current_version
**Commit:** $commit_hash
**Mode:** $([ "$CHECK_ONLY" = true ] && echo "Check Only" || echo "Full Update")

---

## 1. Dependency Audit

### cargo-audit
EOF

    if [[ -f "$REPORT_DIR/audit-output.txt" ]]; then
        echo '```' >> "$REPORT_FILE"
        cat "$REPORT_DIR/audit-output.txt" >> "$REPORT_FILE"
        echo '```' >> "$REPORT_FILE"
    else
        echo "Not run (cargo-audit not installed)" >> "$REPORT_FILE"
    fi

    cat >> "$REPORT_FILE" <<EOF

### Outdated Dependencies
EOF

    if [[ -f "$REPORT_DIR/outdated-deps.txt" ]]; then
        echo '```' >> "$REPORT_FILE"
        head -50 "$REPORT_DIR/outdated-deps.txt" >> "$REPORT_FILE"
        echo '```' >> "$REPORT_FILE"
    fi

    cat >> "$REPORT_FILE" <<EOF

## 2. Insecure Code Patterns

| Pattern | Count | Status |
|---------|-------|--------|
| Hardcoded passwords | $(grep -rn "password\s*=\s*\"" --include="*.rs" src/ 2>/dev/null | wc -l || echo 0) | $([ "$(grep -rn "password\s*=\s*\"" --include="*.rs" src/ 2>/dev/null | wc -l || echo 0)" -eq 0 ] && echo "✅ Pass" || echo "⚠️ Found") |
| Hardcoded API keys | $(grep -rn "api_key\s*=\s*\"" --include="*.rs" src/ 2>/dev/null | wc -l || echo 0) | $([ "$(grep -rn "api_key\s*=\s*\"" --include="*.rs" src/ 2>/dev/null | wc -l || echo 0)" -eq 0 ] && echo "✅ Pass" || echo "⚠️ Found") |
| .unwrap() usage | $(grep -rn "\.unwrap()" --include="*.rs" src/ 2>/dev/null | wc -l || echo 0) | ⚠️ Review recommended |
| unsafe blocks | $(grep -rn "unsafe " --include="*.rs" src/ 2>/dev/null | wc -l || echo 0) | ⚠️ Review required |

## 3. Recommendations

1. ติดตั้ง cargo-audit: \`cargo install cargo-audit\`
2. รัน audit เป็นประจำทุกสัปดาห์
3. อัปเดต dependencies ทุกเดือน
4. ตรวจสอบ unsafe code และ unwrap() usage
5. ใช้ environment variables สำหรับ secrets

---

*Report generated by update-security.sh*
EOF

    log_ok "Report saved to: $REPORT_FILE"
}

# ==============================================================================
# Main Execution
# ==============================================================================
main() {
    echo "========================================"
    echo "  BL1NK Agents - Security Update"
    echo "  $(date -u +'%Y-%m-%d %H:%M:%S UTC')"
    echo "========================================"
    echo ""

    if [[ "$REPORT_ONLY" == true ]]; then
        generate_report
        exit 0
    fi

    # ขั้นตอนที่ 1: ตรวจสอบ vulnerabilities
    check_audit || true

    # ขั้นตอนที่ 2: ตรวจสอบ outdated dependencies
    check_outdated

    # ขั้นตอนที่ 3: อัปเดต dependencies
    update_dependencies

    # ขั้นตอนที่ 4: ตรวจสอบ insecure patterns
    check_insecure_patterns

    # ขั้นตอนที่ 5: สร้าง report
    generate_report

    echo ""
    echo "========================================"
    log_ok "Security update complete!"
    echo "========================================"

    if [[ "$CHECK_ONLY" == false ]]; then
        echo ""
        log_info "ถ้าต้องการ commit การเปลี่ยนแปลง:"
        echo "  git add Cargo.lock"
        echo "  git commit -m 'chore(security): update dependencies'"
    fi
}

main "$@"
