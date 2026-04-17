#!/usr/bin/env bash
# ==============================================================================
# parallel-check.sh - Quality checks for BL1NK Agents Manager
#
# Runs: fmt, clippy, tests, and AGENT VALIDATION.
# Updated: Sequential execution for CI stability.
# ==============================================================================

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

LOG_DIR="target/check-logs"
mkdir -p "$LOG_DIR"

log_info()  { echo -e "${BLUE}[INFO]${NC}  $*"; }
log_ok()    { echo -e "${GREEN}[OK]${NC}    $*"; }
log_error() { echo -e "${RED}[ERROR]${NC} $*"; }

run_fmt() {
    log_info "Checking formatting..."
    if cargo fmt --all --check > "$LOG_DIR/fmt.log" 2>&1; then
        log_ok "cargo fmt: PASS"
        return 0
    else
        log_error "cargo fmt: FAIL"
        return 1
    fi
}

run_clippy() {
    log_info "Running clippy..."
    if cargo clippy --all-targets --all-features -- -D warnings > "$LOG_DIR/clippy.log" 2>&1; then
        log_ok "cargo clippy: PASS"
        return 0
    else
        log_error "cargo clippy: FAIL"
        return 1
    fi
}

run_tests() {
    log_info "Running tests..."
    if cargo test --all-features > "$LOG_DIR/test.log" 2>&1; then
        log_ok "cargo test: PASS"
        return 0
    else
        log_error "cargo test: FAIL"
        return 1
    fi
}

run_agents() {
    log_info "Validating agents..."
    if python3 scripts/validate_agents.py > "$LOG_DIR/agents.log" 2>&1; then
        log_ok "agent-validation: PASS"
        return 0
    else
        log_error "agent-validation: FAIL"
        return 1
    fi
}

main() {
    local f=0 c=0 t=0 a=0
    
    # Run sequentially for CI stability (prevent cargo lock issues)
    run_fmt || f=1
    run_clippy || c=1
    run_tests || t=1
    run_agents || a=1

    echo -e "\n${BOLD}SUMMARY:${NC}"
    [[ $f -eq 0 ]] && echo -e "  ${GREEN}✓${NC} Format" || echo -e "  ${RED}✗${NC} Format"
    [[ $c -eq 0 ]] && echo -e "  ${GREEN}✓${NC} Clippy" || echo -e "  ${RED}✗${NC} Clippy"
    [[ $t -eq 0 ]] && echo -e "  ${GREEN}✓${NC} Tests"  || echo -e "  ${RED}✗${NC} Tests"
    [[ $a -eq 0 ]] && echo -e "  ${GREEN}✓${NC} Agents" || echo -e "  ${RED}✗${NC} Agents"

    if [[ $((f+c+t+a)) -gt 0 ]]; then exit 1; fi
}

main
