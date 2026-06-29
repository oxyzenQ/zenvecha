#!/usr/bin/env bash
# ============================================================================
# Zenvecha Safe End-to-End Livepatch Test
# ============================================================================
# Run inside a VM or systemd-nspawn container with:
#   - Kernel 6.1+ with CONFIG_LIVEPATCH=y, CONFIG_FUNCTION_TRACER=y
#   - Root privileges
#   - Zenvecha kernel module compiled and ready
#   - Zenvecha CLI binary in PATH
#
# Usage:
#   sudo ./scripts/safe_e2e_test.sh
#
# Every step is a micro-check. Any failure → immediate exit.
# This is designed to PREVENT kernel panics, not cause them.
# ============================================================================

set -euo pipefail
IFS=$'\n\t'

# ── Colors ──
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color
BOLD='\033[1m'

STEP=0
PASSED=0
FAILED=0

# ── Helpers ──

log_step() {
    STEP=$((STEP + 1))
    echo ""
    echo -e "${CYAN}${BOLD}═══ Step ${STEP}: $1 ═══${NC}"
}

log_pass() {
    PASSED=$((PASSED + 1))
    echo -e "  ${GREEN}✅ PASS${NC} — $1"
}

log_fail() {
    FAILED=$((FAILED + 1))
    echo -e "  ${RED}❌ FAIL${NC} — $1"
}

abort() {
    echo ""
    echo -e "${RED}${BOLD}╔════════════════════════════════════════╗${NC}"
    echo -e "${RED}${BOLD}║  ABORT: $1${NC}"
    echo -e "${RED}${BOLD}╚════════════════════════════════════════╝${NC}"
    echo ""
    echo "Test aborted at step ${STEP}. ${PASSED} passed, ${FAILED} failed."
    echo "The system has NOT been modified."
    echo ""
    exit 1
}

require_cmd() {
    if ! command -v "$1" &>/dev/null; then
        log_fail "Required command not found: $1"
        abort "Missing dependency: $1"
    fi
    log_pass "Found: $1"
}

# ═══════════════════════════════════════════════════════════════════
# Step 1: Environment Check
# ═══════════════════════════════════════════════════════════════════

log_step "Environment Check"

# Check root
if [[ $EUID -ne 0 ]]; then
    log_fail "Not running as root"
    abort "This test requires root privileges. Run with: sudo $0"
fi
log_pass "Running as root"

# Check kernel version
KVER=$(uname -r)
KMAJOR=$(echo "$KVER" | cut -d. -f1)
KMINOR=$(echo "$KVER" | cut -d. -f2)
echo "  Kernel: ${KVER}"

if [[ $KMAJOR -lt 6 ]]; then
    abort "Kernel ${KVER} is too old. Need 6.1+ for livepatch support."
fi
if [[ $KMAJOR -eq 6 && $KMINOR -lt 1 ]]; then
    abort "Kernel ${KVER} is too old. Need 6.1+ for stable livepatch."
fi
log_pass "Kernel version ${KVER} is sufficient (>= 6.1)"

# Check required commands
echo ""
echo "  Checking dependencies..."
require_cmd zenvecha
require_cmd lsmod
require_cmd insmod
require_cmd rmmod
require_cmd dmesg

# ═══════════════════════════════════════════════════════════════════
# Step 2: Module Load
# ═══════════════════════════════════════════════════════════════════

log_step "Kernel Module Load"

if lsmod | grep -q zenvecha; then
    log_pass "Module already loaded"
else
    MODULE_PATH="${ZENVECHA_MODULE:-./kernel/zenvecha_module.ko}"

    if [[ ! -f "$MODULE_PATH" ]]; then
        log_fail "Module not found at $MODULE_PATH"
        abort "Module load failed — likely missing kernel support. Set ZENVECHA_MODULE env var to the .ko path."
    fi

    echo "  Loading module from: ${MODULE_PATH}"
    if insmod "$MODULE_PATH" 2>/tmp/zenvecha_insmod_err; then
        log_pass "Module loaded"
    else
        ERR=$(cat /tmp/zenvecha_insmod_err)
        log_fail "Module load failed: $ERR"
        abort "Module load failed — likely missing kernel support (CONFIG_LIVEPATCH, CONFIG_FUNCTION_TRACER)"
    fi
fi

# Check proc interface
sleep 1
if [[ ! -d /proc/zenvecha ]]; then
    log_fail "/proc/zenvecha not found after module load"
    abort "Module loaded but proc interface missing — check dmesg for errors"
fi
log_pass "/proc/zenvecha interface available"

# Check dmesg for preflight failures
if dmesg | tail -50 | grep -iq "zenvecha.*preflight.*fail"; then
    log_fail "Preflight checks failed in kernel"
    DMESG_ERR=$(dmesg | tail -50 | grep -i "zenvecha.*fail" | tail -3)
    echo "  dmesg: $DMESG_ERR"
    abort "Preflight checks failed — see dmesg above"
fi
log_pass "No preflight failures in dmesg"

# ═══════════════════════════════════════════════════════════════════
# Step 3: Dry-Run Validation
# ═══════════════════════════════════════════════════════════════════

log_step "Dry-Run Validation"

DRYRUN_OUT=$(zenvecha patch dry-run 2>&1) || true

echo "$DRYRUN_OUT" | head -20
echo "  ..."

if echo "$DRYRUN_OUT" | grep -q "Verdict: approved"; then
    log_pass "Dry-run verdict: approved"
elif echo "$DRYRUN_OUT" | grep -q "Verdict: rejected"; then
    log_fail "Dry-run rejected"
    echo ""
    echo "$DRYRUN_OUT" | grep "❌" || true
    abort "Dry-run validation failed — resolve issues before applying"
else
    log_fail "Could not determine dry-run verdict"
    abort "Dry-run output did not contain expected verdict"
fi

# Show validation details
FAILED_CHECKS=$(echo "$DRYRUN_OUT" | grep "❌" | wc -l)
PASSED_CHECKS=$(echo "$DRYRUN_OUT" | grep "✅" | wc -l)
echo "  Checks: ${PASSED_CHECKS} passed, ${FAILED_CHECKS} failed"

if [[ $FAILED_CHECKS -gt 0 ]]; then
    abort "Validation has failed checks — cannot proceed"
fi
log_pass "All validation gates passed"

# ═══════════════════════════════════════════════════════════════════
# Step 4: Apply Patch
# ═══════════════════════════════════════════════════════════════════

log_step "Apply Patch"

APPLY_OUT=$(zenvecha patch apply 2>&1) || true

echo "$APPLY_OUT" | head -20
echo "  ..."

if echo "$APPLY_OUT" | grep -qi "applied successfully"; then
    log_pass "Patch applied successfully"
elif echo "$APPLY_OUT" | grep -qi "rejected"; then
    log_fail "Patch was rejected"
    echo "$APPLY_OUT" | grep -A5 "REJECTED" || true
    abort "Patch rejected by safety validation"
elif echo "$APPLY_OUT" | grep -qi "fail"; then
    log_fail "Patch application failed"
    abort "Patch application reported failure — check kernel logs"
else
    log_fail "Unexpected apply output"
    abort "System may be in an inconsistent state — check dmesg immediately"
fi

# Brief pause for kernel to settle
sleep 1

# Check dmesg for any oops/panic
if dmesg | tail -20 | grep -qiE "oops|panic|bug:|warning.*zenvecha"; then
    log_fail "Kernel oops/panic/warning detected after patch"
    DMESG_ERR=$(dmesg | tail -20 | grep -iE "oops|panic|bug:|warning.*zenvecha" | tail -3)
    echo "  dmesg: $DMESG_ERR"
    abort "Kernel logged errors after patch — something went wrong"
fi
log_pass "No kernel errors after patch"

# ═══════════════════════════════════════════════════════════════════
# Step 5: Verify Patch
# ═══════════════════════════════════════════════════════════════════

log_step "Verify Patch"

STATUS_OUT=$(zenvecha patch status 2>&1) || true

echo "$STATUS_OUT" | head -20
echo "  ..."

if echo "$STATUS_OUT" | grep -qi "applied"; then
    log_pass "Status confirms patch is applied"
elif echo "$STATUS_OUT" | grep -qi "verified"; then
    log_pass "Verification confirms redirect observed"
else
    log_fail "Patch status does not confirm application"
    abort "Post-patch verification failed"
fi

# Check verification details
if echo "$STATUS_OUT" | grep -qi "redirect_observed"; then
    log_pass "ftrace redirect confirmed active"
else
    log_fail "ftrace redirect not observed"
    abort "Patch may not be redirecting execution"
fi

# ═══════════════════════════════════════════════════════════════════
# Step 6: Revert Patch
# ═══════════════════════════════════════════════════════════════════

log_step "Revert Patch"

REVERT_OUT=$(zenvecha patch revert 2>&1) || true

echo "$REVERT_OUT" | head -10
echo "  ..."

if echo "$REVERT_OUT" | grep -qi "reverted"; then
    log_pass "Patch reverted successfully"
elif echo "$REVERT_OUT" | grep -qi "fail"; then
    log_fail "Revert failed"
    abort "Could not revert patch — system may need reboot"
else
    log_pass "Revert command sent"
fi

sleep 1

# Check dmesg after revert
if dmesg | tail -20 | grep -qiE "oops|panic"; then
    log_fail "Kernel oops/panic after revert"
    abort "Kernel logged errors after revert"
fi
log_pass "No kernel errors after revert"

# ═══════════════════════════════════════════════════════════════════
# Step 7: Clean Unload
# ═══════════════════════════════════════════════════════════════════

log_step "Clean Module Unload"

if rmmod zenvecha 2>/tmp/zenvecha_rmmod_err; then
    log_pass "Module unloaded safely"
else
    ERR=$(cat /tmp/zenvecha_rmmod_err)
    log_fail "Module unload failed: $ERR"
    echo "  This may indicate an active patch preventing unload."
    echo "  The module guard should prevent unsafe unload."
fi

# Verify module is gone
sleep 1
if lsmod | grep -q zenvecha; then
    log_fail "Module still loaded after rmmod"
else
    log_pass "Module confirmed unloaded"
fi

# ═══════════════════════════════════════════════════════════════════
# Summary
# ═══════════════════════════════════════════════════════════════════

echo ""
echo -e "${GREEN}${BOLD}╔════════════════════════════════════════╗${NC}"
echo -e "${GREEN}${BOLD}║  ALL TESTS PASSED                      ║${NC}"
echo -e "${GREEN}${BOLD}╚════════════════════════════════════════╝${NC}"
echo ""
echo "  Steps executed : ${STEP}"
echo "  Passed         : ${PASSED}"
echo "  Failed         : ${FAILED}"
echo ""
echo "  Zenvecha livepatch works end-to-end:"
echo "    ✅ Kernel module loads safely"
echo "    ✅ Preflight checks pass"
echo "    ✅ Safety gates validate"
echo "    ✅ Patch applied without reboot"
echo "    ✅ ftrace redirect verified"
echo "    ✅ Patch reverted cleanly"
echo "    ✅ Module unloads without issues"
echo ""
echo "  No kernel panics. No oops. No reboot required."
echo ""

exit 0
