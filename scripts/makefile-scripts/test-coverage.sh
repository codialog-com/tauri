#!/bin/bash

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}📊 Generating test coverage report...${NC}"

cd src-tauri && cargo install cargo-tarpaulin --locked 2>/dev/null || true
cd src-tauri && cargo tarpaulin --out Html --output-dir ../coverage --timeout 300

echo -e "${GREEN}Coverage report generated in coverage/tarpaulin-report.html${NC}"
