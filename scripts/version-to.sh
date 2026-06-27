#!/usr/bin/env bash
# Copyright (C) 2026 rezky_nightky
# SPDX-License-Identifier: GPL-3.0-only
#
# version-to.sh — Bump Zenvecha version across all files.
# Usage: ./scripts/version-to.sh v0.1.0

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

if [ $# -ne 1 ]; then
    echo "Usage: $0 <new-version>"
    echo "Example: $0 v0.1.0"
    exit 1
fi

NEW_VERSION="$1"
# Strip leading 'v' if present for Cargo.toml
CARGO_VERSION="${NEW_VERSION#v}"

OLD_VERSION=$(grep -m1 '^version' Cargo.toml | sed 's/.*"\(.*\)".*/\1/')

echo "Bumping version: v${OLD_VERSION} → ${NEW_VERSION}"
echo ""

# Cargo.toml
if grep -q "^version = \"${OLD_VERSION}\"" Cargo.toml; then
    sed -i "s/^version = \"${OLD_VERSION}\"/version = \"${CARGO_VERSION}\"/" Cargo.toml
    echo -e "${GREEN}[OK]${NC} Cargo.toml"
else
    echo -e "${RED}[SKIP]${NC} Cargo.toml (version not found)"
fi

# README.md (common version references)
if grep -q "v${OLD_VERSION}" README.md 2>/dev/null; then
    sed -i "s/v${OLD_VERSION}/${NEW_VERSION}/g" README.md
    echo -e "${GREEN}[OK]${NC} README.md"
fi

echo ""
echo -e "${GREEN}Version bump complete: ${NEW_VERSION}${NC}"
echo "Review changes and commit."
