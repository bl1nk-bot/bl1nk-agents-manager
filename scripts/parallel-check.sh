#!/usr/bin/env bash
# ==============================================================================
# parallel-check.sh - Quality checks for BL1NK Agents Manager
#
# Runs: fmt, clippy, tests, and AGENT VALIDATION in parallel.
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
RUN_AGENTS=true
VERBOSE=false
FIX_MODE=false
LOG_DIR="target/check-logs"

usage() {
    cat <<EOF
Usage: $(basename "$0") [OPTIONS]
Options:
  --fmt       Run fmt only
  --clippy    Run clippy only
  --test      Run tests only
  --agents    Run agent validation only
  --verbose   Show output in realtime
  --fix       Run fmt + clippy --fix
EOF
    exit 0
}

log_info()  { echo -e "${BLUE}[INFO]${NC}  $*"; }
log_ok()    { echo -e "${GREEN}[OK]${NC}    $*"; }
log_error() { echo -e "${RED}[ERROR]${NC} $*"; }
log_step()  { echo -e "${CYAN}[STEP]${NC}  $*"; }

while [[ $# -gt 0 ]]; do
    case "$1" in
        --fmt) RUN_CLIPPY=false; RUN_TEST=false; RUN_AGENTS=false; shift ;;
        --clippy) RUN_FMT=false; RUN_TEST=false; RUN_AGENTS=false; shift ;;
        --test) RUN_FMT=false; RUN_CLIPPY=false; RUN_AGENTS=false; shift ;;
        --agents) RUN_FMT=false; RUN_CLIPPY=false; RUN_TEST=false; shift ;;
        --verbose) VERBOSE=true; shift ;;
        --fix) FIX_MODE=true; shift ;;
        *) shift ;;
    esac
done

mkdir -p "$LOG_DIR"

run_fmt() {
    local log="$LOG_DIR/fmt.log"
    local cmd="cargo fmt --all --check"
    [[ "$FIX_MODE" == true ]] && cmd="cargo fmt --all"
    if $cmd > "$log" 2>&1; then log_ok "cargo fmt: PASS"; return 0; else log_error "cargo fmt: FAIL"; return 1; fi
}

run_clippy() {
    local log="$LOG_DIR/clippy.log"
    local cmd="cargo clippy --all-features -- -D warnings"
    [[ "$FIX_MODE" == true ]] && cmd="cargo clippy --all-features --fix --allow-dirty --allow-staged -- -D warnings"
    if $cmd > "$log" 2>&1; then log_ok "cargo clippy: PASS"; return 0; else log_error "cargo clippy: FAIL"; return 1; fi
}

run_tests() {
    local log="$LOG_DIR/test.log"
    if cargo test --all-features > "$log" 2>&1; then log_ok "cargo test: PASS"; return 0; else log_error "cargo test: FAIL"; return 1; fi
}

run_agents() {
    local log="$LOG_DIR/agents.log"
    if python3 scripts/validate_agents.py > "$log" 2>&1; then log_ok "agent-validation: PASS"; return 0; else log_error "agent-validation: FAIL"; return 1; fi
}

main() {
    log_info "Starting parallel checks..."
    local f=0 c=0 t=0 a=0
    
    run_fmt & local pid_f=$!
    run_clippy & local pid_c=$!
    run_tests & local pid_t=$!
    run_agents & local pid_a=$!

    wait $pid_f || f=1
    wait $pid_c || c=1
    wait $pid_t || t=1
    wait $pid_a || a=1

    echo -e "\n${BOLD}SUMMARY:${NC}"
    [[ "$RUN_FMT" == true ]] && ( [[ $f -eq 0 ]] && echo -e "  ${GREEN}✓${NC} Format" || echo -e "  ${RED}✗${NC} Format" )
    [[ "$RUN_CLIPPY" == true ]] && ( [[ $c -eq 0 ]] && echo -e "  ${GREEN}✓${NC} Clippy" || echo -e "  ${RED}✗${NC} Clippy" )
    [[ "$RUN_TEST" == true ]] && ( [[ $t -eq 0 ]] && echo -e "  ${GREEN}✓${NC} Tests" || echo -e "  ${RED}✗${NC} Tests" )
    [[ "$RUN_AGENTS" == true ]] && ( [[ $a -eq 0 ]] && echo -e "  ${GREEN}✓${NC} Agents" || echo -e "  ${RED}✗${NC} Agents" )

    if [[ $((f+c+t+a)) -gt 0 ]]; then exit 1; fi
}

main
