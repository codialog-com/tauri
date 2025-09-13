#!/bin/bash

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}Cleaning application data...${NC}"

rm -rf data/uploads/* data/sessions/* data/logs/* 2>/dev/null || true
rm -rf src-tauri/data/uploads/* src-tauri/data/sessions/* src-tauri/data/logs/* 2>/dev/null || true

echo -e "${GREEN}Application data cleaned${NC}"
