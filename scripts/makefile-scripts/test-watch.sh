#!/bin/bash

# Colors for output
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}👀 Running tests in watch mode...${NC}"

cd src-tauri && cargo install cargo-watch --locked 2>/dev/null || true
cd src-tauri && cargo watch -x test
