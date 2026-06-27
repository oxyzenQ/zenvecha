#!/usr/bin/env bash
# Copyright (C) 2026 rezky_nightky
# SPDX-License-Identifier: GPL-3.0-only
#
# uninstall.sh — Remove Zenvecha from the system.
# Usage:
#   ./scripts/uninstall.sh           # User uninstall (~/.local/bin)
#   ./scripts/uninstall.sh --system  # System uninstall (/usr/local/bin, needs sudo)

set -euo pipefail

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

SYSTEM=false
if [ "${1:-}" = "--system" ]; then
    SYSTEM=true
fi

if $SYSTEM; then
    echo -e "${YELLOW}Removing system-wide install (requires sudo)...${NC}"
    if [ -f /usr/local/bin/zenvecha ]; then
        sudo rm -f /usr/local/bin/zenvecha
        echo -e "${GREEN}Removed /usr/local/bin/zenvecha${NC}"
    else
        echo -e "${RED}/usr/local/bin/zenvecha not found${NC}"
    fi
else
    TARGET="${HOME}/.local/bin/zenvecha"
    if [ -f "${TARGET}" ]; then
        rm -f "${TARGET}"
        echo -e "${GREEN}Removed ${TARGET}${NC}"
    else
        echo -e "${RED}${TARGET} not found${NC}"
    fi
fi

echo -e "${GREEN}Uninstall complete.${NC}"
