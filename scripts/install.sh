#!/usr/bin/env bash
# Copyright (C) 2026 rezky_nightky
# SPDX-License-Identifier: GPL-3.0-only
#
# install.sh — Install Zenvecha binary.
# Usage:
#   ./scripts/install.sh           # User install (~/.local/bin)
#   ./scripts/install.sh --system  # System install (/usr/local/bin, needs sudo)

set -euo pipefail

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

SYSTEM=false
if [ "${1:-}" = "--system" ]; then
    SYSTEM=true
fi

echo -e "${YELLOW}Building Zenvecha...${NC}"
cargo build --release --locked

if $SYSTEM; then
    echo -e "${YELLOW}Installing system-wide (requires sudo)...${NC}"
    sudo install -Dm755 target/release/zenvecha /usr/local/bin/zenvecha
    echo -e "${GREEN}Installed to /usr/local/bin/zenvecha${NC}"
else
    INSTALL_DIR="${HOME}/.local/bin"
    mkdir -p "${INSTALL_DIR}"
    install -Dm755 target/release/zenvecha "${INSTALL_DIR}/zenvecha"
    echo -e "${GREEN}Installed to ${INSTALL_DIR}/zenvecha${NC}"
    if ! echo "${PATH}" | grep -q "${INSTALL_DIR}"; then
        echo -e "${YELLOW}Note: ${INSTALL_DIR} may not be in your PATH.${NC}"
        echo "Add this to your shell profile:"
        echo "  export PATH=\"\${HOME}/.local/bin:\${PATH}\""
    fi
fi

echo -e "${GREEN}Done. Run 'zenvecha --version' to verify.${NC}"
