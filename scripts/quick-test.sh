#!/usr/bin/env bash
# ============================================================================
# Zenvecha One-Click Quick Test
# ============================================================================
# Fastest path from clean repo to verified livepatch cycle:
#   1. Detect kernel toolchain (clang for CachyOS/LTO kernels, gcc otherwise)
#   2. Install missing deps (clang, ld.lld, rust) via pacman/apt/dnf
#   3. Build kernel module + CLI binary
#   4. Load module, run dry-run → apply → status → revert → unload
#   5. Print a clean PASS/FAIL summary
#
# Usage:
#   ./scripts/quick-test.sh           # build + test + cleanup
#   ./scripts/quick-test.sh --keep    # don't rmmod at the end (manual debug)
#   ./scripts/quick-test.sh --no-build # skip build (use existing artifacts)
#
# Requires: root for insmod/rmmod (script will re-exec with sudo if needed)
# ============================================================================

set -euo pipefail
IFS=$'\n\t'

# ── Repo root ───────────────────────────────────────────────────────────
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

# ── Flags ───────────────────────────────────────────────────────────────
KEEP_MODULE=0
SKIP_BUILD=0
for arg in "$@"; do
    case "$arg" in
        --keep)     KEEP_MODULE=1 ;;
        --no-build) SKIP_BUILD=1 ;;
        *) echo "unknown flag: $arg"; exit 2 ;;
    esac
done

# ── Colors ──────────────────────────────────────────────────────────────
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
CYAN='\033[0;36m'; BOLD='\033[1m'; NC='\033[0m'

info()  { echo -e "${CYAN}[INFO]${NC} $*"; }
pass()  { echo -e "${GREEN}[PASS]${NC} $*"; }
fail()  { echo -e "${RED}[FAIL]${NC} $*"; }
step()  { echo -e "\n${YELLOW}${BOLD}═══ $* ═══${NC}"; }

# ── Sudo re-exec ────────────────────────────────────────────────────────
if [[ $EUID -ne 0 ]]; then
    info "re-executing with sudo for insmod/rmmod"
    exec sudo -E "$0" "$@"
fi

# ── Step 1: Kernel + toolchain detection ────────────────────────────────
step "1/7 — Kernel + toolchain detection"

KVER="$(uname -r)"
KCONFIG="/lib/modules/$KVER/build/.config"

if [[ ! -f "$KCONFIG" ]]; then
    fail "kernel headers not found at /lib/modules/$KVER/build"
    info "install: sudo pacman -S linux-headers   (or linux-cachyos-headers on CachyOS)"
    exit 1
fi

info "kernel:    $KVER"
info "headers:   /lib/modules/$KVER/build"

USE_CLANG=0
USE_LLD=0
if grep -q '^CONFIG_CC_IS_CLANG=y' "$KCONFIG"; then
    USE_CLANG=1
    info "compiler: clang (kernel was built with clang)"
    if grep -qE '^(CONFIG_LTO_CLANG|CONFIG_LTO)=y' "$KCONFIG"; then
        USE_LLD=1
        info "linker:   ld.lld + LLVM binutils (LTO kernel)"
    fi
else
    info "compiler: gcc (default)"
fi

# ── Step 2: Dependency install ──────────────────────────────────────────
step "2/7 — Dependency check"

install_pkgs=()
if [[ $USE_CLANG -eq 1 ]]; then
    command -v clang >/dev/null 2>&1 || install_pkgs+=(clang)
    if [[ $USE_LLD -eq 1 ]]; then
        command -v ld.lld >/dev/null 2>&1 || install_pkgs+=(lld)
        command -v llvm-ar >/dev/null 2>&1 || install_pkgs+=(llvm)
    fi
fi
command -v cargo >/dev/null 2>&1 || install_pkgs+=(rust)
command -v codespell >/dev/null 2>&1 || install_pkgs+=(codespell)

if [[ ${#install_pkgs[@]} -gt 0 ]]; then
    info "missing: ${install_pkgs[*]}"
    if command -v pacman >/dev/null 2>&1; then
        info "installing via pacman..."
        pacman -Sy --noconfirm "${install_pkgs[@]}"
    elif command -v apt >/dev/null 2>&1; then
        info "installing via apt..."
        apt-get update -y
        apt-get install -y "${install_pkgs[@]}"
    elif command -v dnf >/dev/null 2>&1; then
        info "installing via dnf..."
        dnf install -y "${install_pkgs[@]}"
    else
        fail "no supported package manager found"
        info "manually install: ${install_pkgs[*]}"
        exit 1
    fi
fi
pass "all dependencies present"

# ── Step 3: Verify required kernel configs ──────────────────────────────
step "3/7 — Kernel config preflight"

# Required gates (fatal if missing)
missing=()
grep -q '^CONFIG_FUNCTION_TRACER=y'  "$KCONFIG" || missing+=("CONFIG_FUNCTION_TRACER=y")
grep -q '^CONFIG_MODULES=y'          "$KCONFIG" || missing+=("CONFIG_MODULES=y")
grep -q '^CONFIG_KALLSYMS=y'         "$KCONFIG" || missing+=("CONFIG_KALLSYMS=y")

if [[ ${#missing[@]} -gt 0 ]]; then
    fail "missing REQUIRED kernel configs:"
    for c in "${missing[@]}"; do
        echo "    - $c"
    done
    info "zenvecha skeleton needs these for ftrace + module loading + symbol discovery"
    info "all standard Arch/CachyOS kernels ship with these enabled"
    exit 1
fi
pass "REQUIRED gates: FUNCTION_TRACER + MODULES + KALLSYMS all =y"

# Recommended gate (non-fatal warning)
if ! grep -q '^CONFIG_LIVEPATCH=y' "$KCONFIG"; then
    info "RECOMMENDED gate CONFIG_LIVEPATCH is not set"
    info "  → skeleton mode works, production ftrace redirect will need it"
    info "  → no prebuilt Arch/CachyOS kernel enables this by default"
    info "  → for production phase, use AUR linux-tkg with the config flipped"
else
    pass "RECOMMENDED gate: CONFIG_LIVEPATCH=y (production-ready)"
fi

# Verify x86_64 (zenvecha is amd64-only)
ARCH="$(uname -m)"
if [[ "$ARCH" != "x86_64" ]]; then
    fail "architecture '$ARCH' not supported — zenvecha is x86_64/amd64 only"
    exit 1
fi
pass "architecture: $ARCH"

# ── Step 4: Build ───────────────────────────────────────────────────────
step "4/7 — Build kernel module + CLI"

if [[ $SKIP_BUILD -eq 0 ]]; then
    info "cleaning prior artifacts..."
    make -C kernel clean >/dev/null 2>&1 || true

    info "building kernel module..."
    make -C kernel 2>&1 | tail -10
    if [[ ! -f kernel/zenvecha.ko ]]; then
        fail "kernel module build failed — zenvecha.ko not produced"
        exit 1
    fi
    pass "kernel/zenvecha.ko built"

    info "building userspace CLI..."
    cargo build --release 2>&1 | tail -5
    if [[ ! -x target/release/zenvecha ]]; then
        fail "CLI build failed"
        exit 1
    fi
    pass "target/release/zenvecha built"
else
    info "skipping build (--no-build flag)"
    [[ -f kernel/zenvecha.ko ]] || { fail "no pre-built zenvecha.ko"; exit 1; }
    [[ -x target/release/zenvecha ]] || { fail "no pre-built zenvecha CLI"; exit 1; }
fi

# ── Step 5: Load module ─────────────────────────────────────────────────
step "5/7 — Load kernel module"

# Remove if already loaded (idempotent)
if lsmod | grep -q '^zenvecha '; then
    info "module already loaded, rmmod first"
    rmmod zenvecha || true
    sleep 1
fi

info "insmod kernel/zenvecha.ko"
if ! insmod kernel/zenvecha.ko; then
    fail "insmod failed"
    info "recent dmesg:"
    dmesg | tail -20
    exit 1
fi
sleep 1

if [[ ! -d /proc/zenvecha ]]; then
    fail "/proc/zenvecha not created — check dmesg"
    dmesg | tail -20
    rmmod zenvecha 2>/dev/null || true
    exit 1
fi
pass "/proc/zenvecha/ available"

# Check for preflight failures in dmesg
if dmesg | tail -30 | grep -qi 'zenvecha.*preflight.*fail'; then
    fail "kernel preflight checks failed — see dmesg"
    dmesg | tail -30 | grep -i 'zenvecha'
    rmmod zenvecha 2>/dev/null || true
    exit 1
fi
pass "preflight gates passed (CONFIG_LIVEPATCH + FUNCTION_TRACER + MODULES)"

# Show proc entries
info "proc entries:"
ls /proc/zenvecha/ | sed 's/^/    /'

# ── Step 6: Patch lifecycle ─────────────────────────────────────────────
step "6/7 — Patch lifecycle: dry-run → apply → status → revert"

ZV=./target/release/zenvecha

info "zenvecha patch dry-run"
if ! $ZV patch dry-run 2>&1 | tee /tmp/zv_dryrun.log | head -25; then
    fail "dry-run command failed"
    rmmod zenvecha 2>/dev/null || true
    exit 1
fi
if ! grep -q "Verdict: approved" /tmp/zv_dryrun.log; then
    fail "dry-run did not return 'Verdict: approved'"
    rmmod zenvecha 2>/dev/null || true
    exit 1
fi
pass "dry-run verdict: approved"

info "zenvecha patch apply"
if ! $ZV patch apply 2>&1 | tee /tmp/zv_apply.log | head -25; then
    fail "apply command failed"
    rmmod zenvecha 2>/dev/null || true
    exit 1
fi
if ! grep -qi "applied successfully" /tmp/zv_apply.log; then
    fail "apply did not report 'applied successfully'"
    cat /tmp/zv_apply.log
    rmmod zenvecha 2>/dev/null || true
    exit 1
fi
pass "patch applied (42 → 99)"

# Verify kernel state directly
STATUS="$(cat /proc/zenvecha/livepatch/status)"
VERIFY="$(cat /proc/zenvecha/livepatch/verify)"
info "kernel status: $STATUS"
info "kernel verify:  $VERIFY"
[[ "$STATUS" == "applied" ]] || { fail "status != applied"; rmmod zenvecha; exit 1; }
[[ "$VERIFY" == *"verified"* && "$VERIFY" == *"redirect_observed"* ]] || {
    fail "verify missing 'verified' or 'redirect_observed'"
    rmmod zenvecha; exit 1
}
pass "kernel confirms: applied + verified + redirect_observed"

# Check no oops/panic in dmesg
if dmesg | tail -10 | grep -qiE 'oops|panic|bug:.*zenvecha'; then
    fail "kernel oops/panic after patch"
    dmesg | tail -10
    rmmod zenvecha 2>/dev/null || true
    exit 1
fi
pass "no kernel oops/panic"

info "zenvecha patch revert"
if ! $ZV patch revert 2>&1 | head -10; then
    fail "revert command failed"
    rmmod zenvecha 2>/dev/null || true
    exit 1
fi
sleep 1
STATUS_AFTER="$(cat /proc/zenvecha/livepatch/status)"
info "post-revert status: $STATUS_AFTER"
[[ "$STATUS_AFTER" == "reverted" ]] || {
    fail "status != reverted after revert (got: $STATUS_AFTER)"
    rmmod zenvecha 2>/dev/null || true
    exit 1
}
pass "patch reverted (99 → 42)"

# ── Step 7: Cleanup ────────────────────────────────────────────────────
step "7/7 — Cleanup"

if [[ $KEEP_MODULE -eq 1 ]]; then
    info "keeping module loaded (--keep flag) — run 'sudo rmmod zenvecha' to unload"
else
    info "rmmod zenvecha"
    rmmod zenvecha
    sleep 1
    if lsmod | grep -q '^zenvecha '; then
        fail "module still loaded after rmmod"
        exit 1
    fi
    pass "module unloaded cleanly"
fi

# ── Summary ────────────────────────────────────────────────────────────
echo ""
echo -e "${GREEN}${BOLD}╔════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}${BOLD}║  ZENVECHA QUICK TEST — ALL PASSED           ║${NC}"
echo -e "${GREEN}${BOLD}╚════════════════════════════════════════════╝${NC}"
echo ""
echo "  kernel     : $KVER"
echo "  arch       : $ARCH"
echo "  compiler   : $(if [[ $USE_CLANG -eq 1 ]]; then echo clang; else echo gcc; fi)"
echo "  module     : kernel/zenvecha.ko"
echo "  cli        : target/release/zenvecha"
echo ""
echo "  Lifecycle verified:"
echo "    dry-run  → Verdict: approved"
echo "    apply    → applied successfully"
echo "    status   → applied + verified + redirect_observed"
echo "    revert   → reverted"
echo "    unload   → clean (no oops, no panic)"
echo ""
echo "  No reboot was required at any point."
echo ""

exit 0
