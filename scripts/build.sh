#!/usr/bin/env bash
# Copyright (C) 2026 rezky_nightky
# SPDX-License-Identifier: GPL-3.0-only
#
# build.sh — Gatekeeper script for Zenvecha.
# Runs all quality checks before commit/push.
# Usage: ./scripts/build.sh --check-all

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

pass()  { echo -e "${GREEN}[PASS]${NC} $*"; }
fail()  { echo -e "${RED}[FAIL]${NC} $*"; exit 1; }
info()  { echo -e "${YELLOW}[INFO]${NC} $*"; }
header(){ echo -e "\n${YELLOW}=== $* ===${NC}"; }

check_all() {
    header "Zenvecha Gatekeeper — check-all"

    # 1. Format
    header "1/5 — cargo fmt --check"
    cargo fmt -- --check || fail "Formatting issues found. Run 'cargo fmt'."

    # 2. Clippy
    header "2/5 — cargo clippy"
    cargo clippy --all-targets --all-features -- -D warnings || fail "Clippy warnings."

    # 3. Build
    header "3/5 — cargo build --release --locked"
    cargo build --release --locked || fail "Build failed."

    # 4. Test
    header "4/5 — cargo test"
    cargo test || fail "Tests failed."

    # 5. Codespell
    header "5/5 — codespell"
    if command -v codespell &>/dev/null; then
        codespell \
            --skip='target,.git,LICENSE,*.png,*.jpg,*.svg,*.ico,*.lock' \
            --ignore-words-list='crate,ser,cant,fo,hart,wee,ot,te,ba' \
            . || fail "Spelling issues found."
        pass "codespell clean"
    else
        info "codespell not installed — skipping (pip install codespell)"
    fi

    # Optional: cargo audit (warning only, does not fail)
    if command -v cargo-audit &>/dev/null || cargo audit --version &>/dev/null 2>&1; then
        header "optional — cargo audit"
        cargo audit || info "Audit warnings (non-blocking)"
    else
        info "cargo-audit not installed — skipping"
    fi

    echo ""
    pass "ALL GATEKEEPER CHECKS PASSED — safe to commit."
}

case "${1:-}" in
    --check-all) check_all ;;
    *)
        echo "Usage: $0 --check-all"
        echo ""
        echo "Zenvecha Gatekeeper — runs before commit/push:"
        echo "  cargo fmt --check"
        echo "  cargo clippy"
        echo "  cargo build --release --locked"
        echo "  cargo test"
        echo "  codespell"
        echo "  cargo audit (warning only)"
        exit 1
        ;;
esac
