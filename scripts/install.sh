#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-only
# Copyright (C) 2026 rezky_nightky (oxyzenQ)
#
# Install script for zenvecha.
# Supports --system (system-wide) and --user (default, ~/.local/bin).
# Run WITHOUT sudo: the script escalates via sudo ONLY for the --system install step.

set -euo pipefail

zenvecha="zenvecha"
REPO_URL="https://github.com/oxyzenQ/zenvecha"

usage() {
    cat <<EOF
Usage: $0 [--system|--user]

  --system   Install system-wide to /usr/bin/${zenvecha}
             (script invokes sudo for the install step only)
  --user     Install to ~/.local/bin/${zenvecha}  (default, no sudo)

The build step (cargo build --release --locked) ALWAYS runs as the current user.
EOF
}

MODE="--user"
while [[ $# -gt 0 ]]; do
    case "$1" in
        --system) MODE="--system"; shift ;;
        --user)   MODE="--user";   shift ;;
        -h|--help) usage; exit 0 ;;
        *) echo "error: unknown argument: $1" >&2; usage; exit 2 ;;
    esac
done

# Refuse to run as root — cargo build must run as the current user.
# If run with sudo, cargo build would create root-owned files in target/,
# breaking future `cargo clean` / `cargo build` for the normal user.
# The script uses sudo internally only for the install step in --system mode.
if [[ $EUID -eq 0 ]]; then
    echo "error: do not run this script with sudo." >&2
    echo "  cargo build would run as root, corrupting target/ ownership." >&2
    echo "  Run: $0 --system" >&2
    echo "  The script will use sudo internally only for the install step." >&2
    exit 1
fi

if [[ ! -f Cargo.toml ]]; then
    echo "error: Cargo.toml not found. Run this script from the repo root." >&2
    exit 1
fi

echo ">> [1/3] Building ${zenvecha} (release, locked)"
cargo build --release --locked

BINARY="target/release/${zenvecha}"
if [[ ! -f "${BINARY}" ]]; then
    echo "error: build produced no binary at ${BINARY}" >&2
    exit 1
fi

echo ">> [2/3] Installing ${zenvecha} (${MODE})"

case "${MODE}" in
    --system)
        # Invoked WITHOUT sudo; escalate only for the install step.
        sudo install -Dm755 "${BINARY}" "/usr/bin/${zenvecha}"
        echo "   installed: /usr/bin/${zenvecha}"
        ;;
    --user)
        user_bin="${HOME}/.local/bin"
        mkdir -p "${user_bin}"
        install -Dm755 "${BINARY}" "${user_bin}/${zenvecha}"
        echo "   installed: ${user_bin}/${zenvecha}"
        ;;
esac

echo ">> [3/3] Done."
echo
echo "Next steps:"
case "${MODE}" in
    --system) echo "  - Run: ${zenvecha} --help" ;;
    --user)   echo "  - Ensure ~/.local/bin is on your PATH" ;;
esac
echo "  - Docs: ${REPO_URL}#readme"
