#!/bin/bash

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${YELLOW}Installing Codialog dependencies...${NC}"

if ! command -v cargo >/dev/null 2>&1; then
    echo -e "${RED}Error: Rust/Cargo not found. Install from https://rustup.rs/${NC}"
    exit 1
fi

if ! command -v npm >/dev/null 2>&1; then
    echo -e "${RED}Error: Node.js/npm not found. Install Node.js first${NC}"
    exit 1
fi

npm install
cargo install tauri-cli

echo -e "${GREEN}Dependencies installed successfully!${NC}"
