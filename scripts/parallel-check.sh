#!/usr/bin/env bash
# ==============================================================================
# parallel-check.sh - รัน fmt, clippy, test พร้อมกันแบบ parallel
#
# ปัญหาเดิม: รันทีละขั้นตอนกินเวลานาน
# วิธีแก้: รัน 3 งานพร้อมกัน (fmt --check, clippy, test)
#
# Usage:
#   ./scripts/parallel-check.sh           # รันทั้งหมด พร้อม exit 0 ถ้าผ่าน
#   ./scripts/parallel-check.sh --verbose # แสดง output แบบ realtime
#   ./scripts/parallel-check.sh --fmt     # รัน fmt อย่างเดียว
#   ./scripts/parallel-check.sh --clippy  # รัน clippy อย่างเดียว
#   ./scripts/parallel-check.sh --test    # รัน test อย่างเดียว
#   ./scripts/parallel-check.sh --fix     # รัน fmt + fix clippy
# ==============================================================================

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

# Flags
RUN_FMT=true
RUN_CLIPPY=true
RUN_TEST=true
VERBOSE=false
FIX_MODE=false
LOG_DIR="target/check-logs"
EXIT_CODE=0

usage() {
    cat <<EOF
Usage: $(basename "$0") [OPTIONS]

Options:
  --fmt       รัน fmt อย่างเดียว
  --clippy    รัน clippy อย่างเดียว
  --test      รัน test อย่างเดียว
  --verbose   แสดง output แบบ realtime
  --fix       รัน fmt + fix clippy auto-fix
  --no-color  ปิดสีใน output
  -h, --help  แสดงวิธีใช้

Examples:
  $(basename "$0")                  # รันทั้งหมด
  $(basename "$0") --verbose        # แสดง output realtime
  $(basename "$0") --test           # รัน test อย่างเดียว
  $(basename "$0") --fix            # auto-fix fmt + clippy
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
        --fmt)       RUN_FMT=true;  RUN_CLIPPY=false; RUN_TEST=false; shift ;;
        --clippy)    RUN_CLIPPY=true; RUN_FMT=false; RUN_TEST=false; shift ;;
        --test)      RUN_TEST=true; RUN_FMT=false; RUN_CLIPPY=false; shift ;;
        --verbose)   VERBOSE=true; shift ;;
        --fix)       FIX_MODE=true; shift ;;
        --no-color)  RED=''; GREEN=''; YELLOW=''; BLUE=''; CYAN=''; BOLD=''; NC=''; shift ;;
        -h|--help)   usage ;;
        *)           log_error "Unknown option: $1"; usage ;;
    esac
done

# สร้าง log directory
mkdir -p "$LOG_DIR"

# ==============================================================================
# ฟังก์ชัน: รัน fmt
# ==============================================================================
run_fmt() {
    log_step "Running cargo fmt..."
    local log_file="$LOG_DIR/fmt.log"

    if [[ "$FIX_MODE" == true ]]; then
        # Fix mode: format และบันทึกผล
        if cargo fmt --all 2>&1 | tee "$log_file"; then
            log_ok "cargo fmt: formatted successfully"
            return 0
        else
            log_error "cargo fmt: failed"
            return 1
        fi
    else
        # Check mode: ตรวจสอบโดยไม่แก้ไข
        if cargo fmt --all --check 2>&1 | tee "$log_file"; then
            log_ok "cargo fmt: code is properly formatted"
            return 0
        else
            log_error "cargo fmt: code needs formatting (see $log_file)"
            return 1
        fi
    fi
}

# ==============================================================================
# ฟังก์ชัน: รัน clippy
# ==============================================================================
run_clippy() {
    log_step "Running cargo clippy..."
    local log_file="$LOG_DIR/clippy.log"

    if [[ "$FIX_MODE" == true ]]; then
        # Fix mode: auto-fix suggestions
        if cargo clippy --all-features --fix --allow-dirty --allow-staged -- -D warnings 2>&1 | tee "$log_file"; then
            log_ok "cargo clippy: no warnings (with auto-fix applied)"
            return 0
        else
            log_error "cargo clippy: found warnings (see $log_file)"
            return 1
        fi
    else
        # Check mode
        if cargo clippy --all-features -- -D warnings 2>&1 | tee "$log_file"; then
            log_ok "cargo clippy: no warnings"
            return 0
        else
            log_error "cargo clippy: found warnings (see $log_file)"
            return 1
        fi
    fi
}

# ==============================================================================
# ฟังก์ชัน: รัน tests
# ==============================================================================
run_tests() {
    log_step "Running cargo test..."
    local log_file="$LOG_DIR/test.log"

    if cargo test --all-features 2>&1 | tee "$log_file"; then
        log_ok "cargo test: all tests passed"
        return 0
    else
        log_error "cargo test: some tests failed (see $log_file)"
        return 1
    fi
}

# ==============================================================================
# ฟังก์ชัน: แสดง summary หลังรันเสร็จ
# ==============================================================================
show_summary() {
    local fmt_result="$1"
    local clippy_result="$2"
    local test_result="$3"

    echo ""
    echo -e "${BOLD}========================================${NC}"
    echo -e "${BOLD}  PARALLEL CHECK RESULTS${NC}"
    echo -e "${BOLD}========================================${NC}"
    echo ""

    if [[ "$RUN_FMT" == true ]]; then
        if [[ $fmt_result -eq 0 ]]; then
            echo -e "  ${GREEN}✓${NC} Format   ${GREEN}PASS${NC}"
        else
            echo -e "  ${RED}✗${NC} Format   ${RED}FAIL${NC}  (see $LOG_DIR/fmt.log)"
        fi
    fi

    if [[ "$RUN_CLIPPY" == true ]]; then
        if [[ $clippy_result -eq 0 ]]; then
            echo -e "  ${GREEN}✓${NC} Clippy   ${GREEN}PASS${NC}"
        else
            echo -e "  ${RED}✗${NC} Clippy   ${RED}FAIL${NC}  (see $LOG_DIR/clippy.log)"
        fi
    fi

    if [[ "$RUN_TEST" == true ]]; then
        if [[ $test_result -eq 0 ]]; then
            echo -e "  ${GREEN}✓${NC} Tests    ${GREEN}PASS${NC}"
        else
            echo -e "  ${RED}✗${NC} Tests    ${RED}FAIL${NC}  (see $LOG_DIR/test.log)"
        fi
    fi

    echo ""
    echo -e "${BOLD}========================================${NC}"

    # Determine overall result
    if [[ $fmt_result -eq 0 ]] && [[ $clippy_result -eq 0 ]] && [[ $test_result -eq 0 ]]; then
        echo -e "${GREEN}${BOLD}  ALL CHECKS PASSED ✓${NC}"
        echo -e "${BOLD}========================================${NC}"
        return 0
    else
        echo -e "${RED}${BOLD}  SOME CHECKS FAILED ✗${NC}"
        echo -e "${BOLD}========================================${NC}"
        echo ""
        echo "Log files:"
        [[ "$RUN_FMT" == true ]]     && echo "  - $LOG_DIR/fmt.log"
        [[ "$RUN_CLIPPY" == true ]]  && echo "  - $LOG_DIR/clippy.log"
        [[ "$RUN_TEST" == true ]]    && echo "  - $LOG_DIR/test.log"
        return 1
    fi
}

# ==============================================================================
# Main: รัน parallel checks
# ==============================================================================
main() {
    echo -e "${BOLD}========================================${NC}"
    echo -e "${BOLD}  BL1NK Agents - Parallel Checks${NC}"
    echo -e "${BOLD}  $(date -u +'%Y-%m-%d %H:%M:%S UTC')${NC}"
    echo -e "${BOLD}========================================${NC}"
    echo ""

    if [[ "$FIX_MODE" == true ]]; then
        log_warn "FIX MODE: จะแก้ไข fmt และ clippy auto-fix"
        echo ""
    fi

    local fmt_result=0
    local clippy_result=0
    local test_result=0

    if [[ "$RUN_FMT" == true ]] && [[ "$RUN_CLIPPY" == true ]] && [[ "$RUN_TEST" == true ]]; then
        # ============================================================
        # PARALLEL MODE: รัน 3 งานพร้อมกัน
        # ============================================================
        log_info "Running 3 checks in PARALLEL..."
        echo ""

        # Spawn background processes
        if [[ "$VERBOSE" == true ]]; then
            # Verbose: แสดง output realtime
            run_fmt &
            local fmt_pid=$!

            run_clippy &
            local clippy_pid=$!

            run_tests &
            local test_pid=$!

            # Wait for all
            wait $fmt_pid   || fmt_result=1
            wait $clippy_pid || clippy_result=1
            wait $test_pid   || test_result=1
        else
            # Quiet: เก็บ output ไว้ใน log
            run_fmt > /dev/null 2>&1 &
            local fmt_pid=$!

            run_clippy > /dev/null 2>&1 &
            local clippy_pid=$!

            run_tests > /dev/null 2>&1 &
            local test_pid=$!

            # แสดง progress
            local dots=""
            while kill -0 $fmt_pid 2>/dev/null || kill -0 $clippy_pid 2>/dev/null || kill -0 $test_pid 2>/dev/null; do
                sleep 1
                dots="${dots}."
                if [[ ${#dots} -gt 30 ]]; then
                    echo -ne "\r  Running...${dots}"
                    dots=""
                fi
                echo -ne "\r  Running...${dots}"
            done
            echo ""

            # Collect results
            wait $fmt_pid   || fmt_result=1
            wait $clippy_pid || clippy_result=1
            wait $test_pid   || test_result=1
        fi

    elif [[ "$RUN_FMT" == true ]]; then
        run_fmt || fmt_result=1
    elif [[ "$RUN_CLIPPY" == true ]]; then
        run_clippy || clippy_result=1
    elif [[ "$RUN_TEST" == true ]]; then
        run_tests || test_result=1
    fi

    # แสดง summary
    show_summary $fmt_result $clippy_result $test_result

    # Return exit code (fail ถ้ามีตัวใดตัวหนึ่ง fail)
    if [[ $fmt_result -ne 0 ]] || [[ $clippy_result -ne 0 ]] || [[ $test_result -ne 0 ]]; then
        exit 1
    fi
}

main "$@"
