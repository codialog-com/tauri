#!/bin/bash

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}🧹 Cleaning test artifacts...${NC}"

cd src-tauri && cargo clean
rm -rf coverage/ 2>/dev/null || true

echo -e "${GREEN}Test artifacts cleaned${NC}"
