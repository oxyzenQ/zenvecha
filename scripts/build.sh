#!/usr/bin/env bash
# Copyright (C) 2026 rezky_nightky
# SPDX-License-Identifier: GPL-3.0-only
#
# build.sh — Gatekeeper script for Zenvecha.
# Runs all quality checks before commit/push.
# Usage: ./scripts/build.sh --check-all

set -euo pipefail

# Ensure cargo tools are discoverable
if [ -f "$HOME/.cargo/env" ]; then
    . "$HOME/.cargo/env"
fi

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

pass()  { echo -e "${GREEN}[PASS]${NC} $*"; }
fail()  { echo -e "${RED}[FAIL]${NC} $*"; exit 1; }
info()  { echo -e "${YELLOW}[INFO]${NC} $*"; }
header(){ echo -e "\n${YELLOW}=== $* ===${NC}"; }

require_tool() {
    local name="$1"
    local install_cmd="$2"
    if ! command -v "$name" &>/dev/null; then
        echo -e "${RED}[MISSING]${NC} ${name} is not installed."
        echo ""
        echo "  Install:"
        echo "    ${install_cmd}"
        echo ""
        echo "  ${name} is REQUIRED for Zenvecha gatekeeper."
        echo "  Skipping this check compromises supply chain security."
        exit 1
    fi
}

check_all() {
    header "Zenvecha Gatekeeper — check-all"

    # 1. Format
    header "1/7 — cargo fmt --check"
    cargo fmt -- --check || fail "Formatting issues found. Run 'cargo fmt'."
    pass "cargo fmt"

    # 2. Clippy
    header "2/7 — cargo clippy"
    cargo clippy --all-targets --all-features -- -D warnings || fail "Clippy warnings."
    pass "cargo clippy"

    # 3. Build
    header "3/7 — cargo build --release --locked"
    cargo build --release --locked || fail "Build failed."
    pass "cargo build"

    # 4. Test
    header "4/7 — cargo test"
    cargo test || fail "Tests failed."
    pass "cargo test"

    # 5. Codespell
    header "5/7 — codespell"
    require_tool codespell "pip install codespell"
    codespell \
        --skip='target,.git,LICENSE,*.png,*.jpg,*.svg,*.ico,*.lock' \
        --ignore-words-list='crate,ser,cant,fo,hart,wee,ot,te,ba' \
        . || fail "Spelling issues found. Run 'codespell -w' to auto-fix."
    pass "codespell"

    # 6. Security audit (CVE)
    header "6/7 — cargo audit"
    require_tool cargo-audit "cargo install cargo-audit"
    cargo audit || fail "Security vulnerabilities found. Review and update dependencies."
    pass "cargo audit"

    # 7. License & supply chain check
    header "7/7 — cargo deny check"
    require_tool cargo-deny "cargo install cargo-deny"
    cargo deny check 2>&1 || fail "cargo-deny violations found. Review licenses, sources, and advisories."
    pass "cargo deny"

    echo ""
    pass "ALL GATEKEEPER CHECKS PASSED — safe to commit."
}

case "${1:-}" in
    --check-all) check_all ;;
    *)
        echo "Usage: $0 --check-all"
        echo ""
        echo "Zenvecha Gatekeeper — runs before commit/push:"
        echo ""
        echo "  1. cargo fmt --check"
        echo "  2. cargo clippy"
        echo "  3. cargo build --release --locked"
        echo "  4. cargo test"
        echo "  5. codespell"
        echo "  6. cargo audit     (CVE check — REQUIRED)"
        echo "  7. cargo deny      (license/supply chain — REQUIRED)"
        echo ""
        echo "Prerequisites:"
        echo "  cargo install cargo-audit"
        echo "  cargo install cargo-deny"
        echo "  pip install codespell"
        exit 1
        ;;
esac
